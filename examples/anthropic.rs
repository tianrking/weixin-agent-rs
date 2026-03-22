use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use wechat_rs_sdk::{Agent, Bot, ChatRequest, ChatResponse, LoginOptions, Result, StartOptions};

#[derive(Clone)]
struct AnthropicAgent {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    system_prompt: Option<String>,
    history: Arc<Mutex<HashMap<String, Vec<serde_json::Value>>>>,
}

#[async_trait]
impl Agent for AnthropicAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let mut user_content = Vec::new();
        if !request.text.trim().is_empty() {
            user_content.push(json!({"type":"text","text":request.text}));
        }
        if let Some(media) = &request.media {
            if matches!(media.kind, wechat_rs_sdk::MediaKind::Image) {
                let bytes = tokio::fs::read(&media.file_path).await?;
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
                user_content.push(json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": media.mime_type,
                        "data": b64
                    }
                }));
            }
        }

        let mut history = self.history.lock().await;
        let conv = history.entry(request.conversation_id.clone()).or_default();
        conv.push(json!({"role":"user","content":user_content}));

        let body = json!({
            "model": self.model,
            "max_tokens": 2048,
            "messages": conv,
            "system": self.system_prompt,
        });

        let resp = self
            .client
            .post(format!("{}/v1/messages", self.base_url.trim_end_matches('/')))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let value: serde_json::Value = resp.json().await?;
        if !status.is_success() {
            return Err(wechat_rs_sdk::WechatError::Api(format!("anthropic error {status}: {}", value)));
        }

        let reply = value
            .pointer("/content/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        conv.push(json!({"role":"assistant","content":[{"type":"text","text":reply}]}));

        Ok(ChatResponse {
            text: Some(reply),
            media: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").with_target(false).init();

    if std::env::args().any(|a| a == "login") {
        let account = Bot::login(LoginOptions::default()).await?;
        println!("login success: {account}");
        return Ok(());
    }

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| wechat_rs_sdk::WechatError::Api("ANTHROPIC_API_KEY is required".to_string()))?;
    let base_url = std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    let model = std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
    let system_prompt = std::env::var("SYSTEM_PROMPT").ok();

    let agent = AnthropicAgent {
        client: reqwest::Client::new(),
        api_key,
        base_url,
        model,
        system_prompt,
        history: Arc::new(Mutex::new(HashMap::new())),
    };

    Bot::start(agent, StartOptions::default()).await
}
