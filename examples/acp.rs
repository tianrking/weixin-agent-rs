use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use wechat_rs_sdk::agent::acp::{AcpAgent, AcpAgentOptions};
use wechat_rs_sdk::{Agent, Bot, ChatRequest, ChatResponse, LoginOptions, Result, StartOptions};

struct DelegatingAgent {
    acp: AcpAgent,
}

#[async_trait]
impl Agent for DelegatingAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        self.acp.chat(request).await
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

    let command = std::env::var("ACP_COMMAND").unwrap_or_else(|_| "npx".to_string());
    let args = std::env::var("ACP_ARGS")
        .map(|v| v.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>())
        .unwrap_or_else(|_| vec!["-y".to_string(), "@zed-industries/codex-acp".to_string()]);

    let acp = AcpAgent::new(AcpAgentOptions {
        command,
        args,
        cwd: None,
        env: HashMap::new(),
        prompt_timeout: Duration::from_secs(180),
    })
    .await?;

    Bot::start(DelegatingAgent { acp }, StartOptions::default()).await
}
