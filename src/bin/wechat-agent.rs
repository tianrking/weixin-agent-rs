use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use wechat_rs_sdk::agent::acp::{AcpAgent, AcpAgentOptions};
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
            return Err(wechat_rs_sdk::WechatError::Api(format!("openai error {status}: {}", value)));
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

struct EchoAgent;

#[async_trait]
impl Agent for EchoAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        Ok(ChatResponse {
            text: Some(format!("你说了: {}", request.text)),
            media: None,
        })
    }
}

enum HubAgent {
    Echo(EchoAgent),
    Acp(AcpAgent),
    OpenAI(OpenAIAgent),
    Anthropic(AnthropicAgent),
}

#[async_trait]
impl Agent for HubAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        match self {
            HubAgent::Echo(v) => v.chat(request).await,
            HubAgent::Acp(v) => v.chat(request).await,
            HubAgent::OpenAI(v) => v.chat(request).await,
            HubAgent::Anthropic(v) => v.chat(request).await,
        }
    }
}

#[derive(Debug, Clone)]
struct Cli {
    login: bool,
    agent: String,
    account_id: Option<String>,
    acp_command: Option<String>,
    acp_args: Option<String>,
}

impl Cli {
    fn parse() -> Self {
        let mut cli = Self {
            login: false,
            agent: "codex".to_string(),
            account_id: None,
            acp_command: None,
            acp_args: None,
        };

        let mut args = std::env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "login" | "--login" => cli.login = true,
                "--agent" => {
                    if let Some(v) = args.next() {
                        cli.agent = v;
                    }
                }
                "--account" => {
                    if let Some(v) = args.next() {
                        cli.account_id = Some(v);
                    }
                }
                "--acp-command" => {
                    if let Some(v) = args.next() {
                        cli.acp_command = Some(v);
                    }
                }
                "--acp-args" => {
                    if let Some(v) = args.next() {
                        cli.acp_args = Some(v);
                    }
                }
                "-h" | "--help" => print_help_and_exit(),
                _ => {}
            }
        }
        cli
    }
}

fn print_help_and_exit() -> ! {
    println!(
        "wechat-agent\n\nUSAGE:\n  wechat-agent [login|--login] [--agent <name>] [--account <id>] [--acp-command <cmd>] [--acp-args \"...\"]\n\nAGENTS:\n  claude | codex | openclaw | acp | openai | anthropic | echo\n\nEXAMPLES:\n  wechat-agent --login --agent codex\n  wechat-agent --agent claude --account <account_id>\n  OPENAI_API_KEY=... wechat-agent --agent openai\n"
    );
    std::process::exit(0);
}

fn acp_preset(agent: &str) -> (String, Vec<String>) {
    match agent {
        "claude" => (
            "npx".to_string(),
            vec!["-y".to_string(), "@zed-industries/claude-agent-acp".to_string()],
        ),
        "codex" => (
            "npx".to_string(),
            vec!["-y".to_string(), "@zed-industries/codex-acp".to_string()],
        ),
        "openclaw" => ("openclaw".to_string(), vec!["acp".to_string()]),
        _ => (
            std::env::var("ACP_COMMAND").unwrap_or_else(|_| "npx".to_string()),
            std::env::var("ACP_ARGS")
                .map(|v| v.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_else(|_| vec!["-y".to_string(), "@zed-industries/codex-acp".to_string()]),
        ),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").with_target(false).init();

    let cli = Cli::parse();

    let mut start_account_id = cli.account_id.clone();
    if cli.login {
        let id = Bot::login(LoginOptions::default()).await?;
        println!("login success: {id}");
        if start_account_id.is_none() {
            start_account_id = Some(id);
        }
    }

    let agent_name = cli.agent.to_lowercase();
    let agent = match agent_name.as_str() {
        "echo" => HubAgent::Echo(EchoAgent),
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| wechat_rs_sdk::WechatError::Api("OPENAI_API_KEY is required".to_string()))?;
            let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.4".to_string());
            let system_prompt = std::env::var("SYSTEM_PROMPT").ok();

            HubAgent::OpenAI(OpenAIAgent {
                client: reqwest::Client::new(),
                api_key,
                base_url,
                model,
                system_prompt,
                history: Arc::new(Mutex::new(HashMap::new())),
            })
        }
        "anthropic" => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| wechat_rs_sdk::WechatError::Api("ANTHROPIC_API_KEY is required".to_string()))?;
            let base_url = std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".to_string());
            let model = std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
            let system_prompt = std::env::var("SYSTEM_PROMPT").ok();

            HubAgent::Anthropic(AnthropicAgent {
                client: reqwest::Client::new(),
                api_key,
                base_url,
                model,
                system_prompt,
                history: Arc::new(Mutex::new(HashMap::new())),
            })
        }
        "claude" | "codex" | "openclaw" | "acp" => {
            let (mut command, mut args) = acp_preset(&agent_name);
            if let Some(c) = cli.acp_command.clone() {
                command = c;
            }
            if let Some(a) = cli.acp_args.clone() {
                args = a.split_whitespace().map(|s| s.to_string()).collect();
            }

            println!("starting ACP agent: {} {}", command, args.join(" "));

            HubAgent::Acp(
                AcpAgent::new(AcpAgentOptions {
                    command,
                    args,
                    cwd: None,
                    env: HashMap::new(),
                    prompt_timeout: Duration::from_secs(180),
                })
                .await?,
            )
        }
        other => {
            return Err(wechat_rs_sdk::WechatError::Api(format!(
                "unsupported --agent: {} (supported: claude|codex|openclaw|acp|openai|anthropic|echo)",
                other
            )));
        }
    };

    Bot::start(
        agent,
        StartOptions {
            account_id: start_account_id,
        },
    )
    .await
}
