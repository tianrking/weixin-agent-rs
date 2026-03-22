use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use wechat_rs_sdk::{Agent, Bot, ChatRequest, ChatResponse, LoginOptions, Result, StartOptions};

#[derive(Clone)]
struct OpenAIAgent {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    system_prompt: Option<String>,
    history: Arc<Mutex<HashMap<String, Vec<serde_json::Value>>>>,
}

#[async_trait]
impl Agent for OpenAIAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let mut msg = Vec::new();
        if !request.text.trim().is_empty() {
            msg.push(json!({"type":"text","text":request.text}));
        }
        if let Some(media) = &request.media {
            if matches!(media.kind, wechat_rs_sdk::MediaKind::Image) {
                let bytes = tokio::fs::read(&media.file_path).await?;
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
                msg.push(json!({"type":"image_url","image_url":{"url":format!("data:{};base64,{}", media.mime_type, b64)}}));
            }
        }

        let mut history = self.history.lock().await;
        let conv = history.entry(request.conversation_id.clone()).or_default();
        conv.push(json!({"role":"user","content":msg}));

        let mut messages = Vec::new();
        if let Some(system) = &self.system_prompt {
            messages.push(json!({"role":"system","content":system}));
        }
        messages.extend(conv.clone());

        let body = json!({
            "model": self.model,
            "messages": messages,
        });

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let value: serde_json::Value = resp.json().await?;
        if !status.is_success() {
            let msg = value.to_string();
            return Err(wechat_rs_sdk::WechatError::Api(format!("openai error {status}: {msg}")));
        }

        let reply = value
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        conv.push(json!({"role":"assistant","content":reply}));

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

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| wechat_rs_sdk::WechatError::Api("OPENAI_API_KEY is required".to_string()))?;
    let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.4".to_string());
    let system_prompt = std::env::var("SYSTEM_PROMPT").ok();

    let agent = OpenAIAgent {
        client: reqwest::Client::new(),
        api_key,
        base_url,
        model,
        system_prompt,
        history: Arc::new(Mutex::new(HashMap::new())),
    };

    Bot::start(agent, StartOptions::default()).await
}
