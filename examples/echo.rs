use async_trait::async_trait;
use wechat_rs_sdk::{Agent, Bot, ChatRequest, ChatResponse, LoginOptions, Result, StartOptions};

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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    if std::env::args().any(|a| a == "login") {
        let account_id = Bot::login(LoginOptions::default()).await?;
        println!("login success: {}", account_id);
        return Ok(());
    }

    Bot::start(EchoAgent, StartOptions::default()).await
}
