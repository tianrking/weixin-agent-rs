use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::agent::acp::{AcpAgent, AcpAgentOptions};
use crate::agent::{Agent, ChatRequest, ChatResponse, MediaKind};
use crate::error::{Result, WechatError};
use crate::space::SpaceConfig;

#[derive(Clone)]
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
            if matches!(media.kind, MediaKind::Image) {
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

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .json(&json!({"model": self.model, "messages": messages}))
            .send()
            .await?;

        let status = resp.status();
        let value: serde_json::Value = resp.json().await?;
        if !status.is_success() {
            return Err(WechatError::Api(format!("openai error {status}: {}", value)));
        }

        let reply = value
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        conv.push(json!({"role":"assistant","content":reply}));
        Ok(ChatResponse { text: Some(reply), media: None })
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
            if matches!(media.kind, MediaKind::Image) {
                let bytes = tokio::fs::read(&media.file_path).await?;
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
                user_content.push(json!({
                    "type":"image",
                    "source":{"type":"base64","media_type":media.mime_type,"data":b64}
                }));
            }
        }

        let mut history = self.history.lock().await;
        let conv = history.entry(request.conversation_id.clone()).or_default();
        conv.push(json!({"role":"user","content":user_content}));

        let resp = self
            .client
            .post(format!("{}/v1/messages", self.base_url.trim_end_matches('/')))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.model,
                "max_tokens": 2048,
                "messages": conv,
                "system": self.system_prompt,
            }))
            .send()
            .await?;

        let status = resp.status();
        let value: serde_json::Value = resp.json().await?;
        if !status.is_success() {
            return Err(WechatError::Api(format!("anthropic error {status}: {}", value)));
        }

        let reply = value
            .pointer("/content/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        conv.push(json!({"role":"assistant","content":[{"type":"text","text":reply}]}));
        Ok(ChatResponse { text: Some(reply), media: None })
    }
}

#[derive(Clone)]
struct CliCodexAgent {
    command: String,
    base_args: Vec<String>,
    system_prompt: Option<String>,
    history: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

#[async_trait]
impl Agent for CliCodexAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let prompt = {
            let mut history = self.history.lock().await;
            let conv = history.entry(request.conversation_id.clone()).or_default();
            let mut prompt = String::new();
            if let Some(system) = &self.system_prompt {
                prompt.push_str("System:\n");
                prompt.push_str(system);
                prompt.push_str("\n\n");
            }
            if !conv.is_empty() {
                prompt.push_str("Conversation so far:\n");
                prompt.push_str(&conv.join("\n"));
                prompt.push_str("\n\n");
            }
            prompt.push_str("User:\n");
            prompt.push_str(&request.text);
            conv.push(format!("User: {}", request.text));
            prompt
        };

        let out_path = std::env::temp_dir()
            .join("wechat-rs-sdk")
            .join("codex-cli")
            .join(format!("{}.txt", uuid::Uuid::new_v4()));
        if let Some(parent) = out_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut args = self.base_args.clone();
        args.push("--output-last-message".to_string());
        args.push(out_path.to_string_lossy().to_string());
        args.push(prompt);

        let output = Command::new(&self.command).args(&args).output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Err(WechatError::Api(format!(
                "codex cli failed: {}",
                if !stderr.is_empty() { stderr } else { stdout }
            )));
        }

        let reply = match tokio::fs::read_to_string(&out_path).await {
            Ok(v) if !v.trim().is_empty() => v,
            _ => String::from_utf8_lossy(&output.stdout).to_string(),
        };
        let reply = reply.trim().to_string();

        let mut history = self.history.lock().await;
        history
            .entry(request.conversation_id)
            .or_default()
            .push(format!("Assistant: {}", reply));

        Ok(ChatResponse { text: Some(reply), media: None })
    }
}

#[derive(Clone)]
enum ManagedAgent {
    Echo(EchoAgent),
    Acp(AcpAgent),
    OpenAI(OpenAIAgent),
    Anthropic(AnthropicAgent),
    CliCodex(CliCodexAgent),
}

#[async_trait]
impl Agent for ManagedAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        match self {
            ManagedAgent::Echo(v) => v.chat(request).await,
            ManagedAgent::Acp(v) => v.chat(request).await,
            ManagedAgent::OpenAI(v) => v.chat(request).await,
            ManagedAgent::Anthropic(v) => v.chat(request).await,
            ManagedAgent::CliCodex(v) => v.chat(request).await,
        }
    }
}

pub struct SpaceAgentRouter {
    default_agent: String,
    bindings: HashMap<String, String>,
    agents: HashMap<String, Arc<ManagedAgent>>,
}

impl SpaceAgentRouter {
    pub async fn new(space: &SpaceConfig) -> Result<Self> {
        let mut names = Vec::new();
        names.push(space.agent.clone());
        for agent in space.user_bindings.values() {
            if !names.contains(agent) {
                names.push(agent.clone());
            }
        }

        let mut agents = HashMap::new();
        for name in names {
            agents.insert(name.clone(), Arc::new(build_agent(&name).await?));
        }

        Ok(Self {
            default_agent: space.agent.clone(),
            bindings: space.user_bindings.clone().into_iter().collect(),
            agents,
        })
    }
}

#[async_trait]
impl Agent for SpaceAgentRouter {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let agent_name = self
            .bindings
            .get(&request.conversation_id)
            .cloned()
            .unwrap_or_else(|| self.default_agent.clone());
        let agent = self
            .agents
            .get(&agent_name)
            .ok_or_else(|| WechatError::Api(format!("agent not initialized: {agent_name}")))?;
        agent.chat(request).await
    }
}

async fn build_agent(name: &str) -> Result<ManagedAgent> {
    let name = name.trim().to_lowercase();
    match name.as_str() {
        "echo" => Ok(ManagedAgent::Echo(EchoAgent)),
        "openai" => Ok(ManagedAgent::OpenAI(OpenAIAgent {
            client: reqwest::Client::new(),
            api_key: std::env::var("OPENAI_API_KEY")
                .map_err(|_| WechatError::Api("OPENAI_API_KEY is required".to_string()))?,
            base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string()),
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.4".to_string()),
            system_prompt: std::env::var("SYSTEM_PROMPT").ok(),
            history: Arc::new(Mutex::new(HashMap::new())),
        })),
        "anthropic" => Ok(ManagedAgent::Anthropic(AnthropicAgent {
            client: reqwest::Client::new(),
            api_key: std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| WechatError::Api("ANTHROPIC_API_KEY is required".to_string()))?,
            base_url: std::env::var("ANTHROPIC_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            model: std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            system_prompt: std::env::var("SYSTEM_PROMPT").ok(),
            history: Arc::new(Mutex::new(HashMap::new())),
        })),
        "codex" | "claude" | "openclaw" => build_local_agent(&name).await,
        other => Err(WechatError::Api(format!("unsupported agent: {other}"))),
    }
}

async fn build_local_agent(name: &str) -> Result<ManagedAgent> {
    let (command, args) = local_agent_command(name);
    let resolved = resolve_spawn_command(&command, &args);
    if resolved.available {
        if let Ok(agent) = AcpAgent::new(AcpAgentOptions {
            command: resolved.command.clone(),
            args: resolved.args.clone(),
            cwd: None,
            env: HashMap::new(),
            prompt_timeout: Duration::from_secs(180),
        })
        .await
        {
            return Ok(ManagedAgent::Acp(agent));
        }
    }

    if name == "codex" {
        if let Some((command, args)) = native_codex_command() {
            return Ok(ManagedAgent::CliCodex(CliCodexAgent {
                command,
                base_args: args,
                system_prompt: std::env::var("SYSTEM_PROMPT").ok(),
                history: Arc::new(Mutex::new(HashMap::new())),
            }));
        }
    }

    Err(WechatError::Api(format!("failed to initialize local agent: {name}")))
}

fn local_agent_command(name: &str) -> (String, Vec<String>) {
    match name {
        "claude" => ("npx".to_string(), vec!["-y".to_string(), "@zed-industries/claude-agent-acp".to_string()]),
        "codex" => ("npx".to_string(), vec!["-y".to_string(), "@zed-industries/codex-acp".to_string()]),
        "openclaw" => ("openclaw".to_string(), vec!["acp".to_string()]),
        _ => ("npx".to_string(), vec!["-y".to_string(), "@zed-industries/codex-acp".to_string()]),
    }
}

#[derive(Debug, Clone)]
struct SpawnSpec {
    available: bool,
    command: String,
    args: Vec<String>,
}

#[cfg(windows)]
const POWERSHELL_EXE: &str = "C:\\WINDOWS\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";

fn resolve_spawn_command(command: &str, args: &[String]) -> SpawnSpec {
    #[cfg(windows)]
    {
        if let Some(path) = resolve_command_in_path(command) {
            let ext = Path::new(&path)
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if ext == "cmd" || ext == "bat" || ext == "ps1" {
                let mut expr = format!("& '{}'", path.replace('\'', "''"));
                for arg in args {
                    expr.push(' ');
                    expr.push('\'');
                    expr.push_str(&arg.replace('\'', "''"));
                    expr.push('\'');
                }
                return SpawnSpec {
                    available: true,
                    command: POWERSHELL_EXE.to_string(),
                    args: vec![
                        "-NoLogo".to_string(),
                        "-NoProfile".to_string(),
                        "-ExecutionPolicy".to_string(),
                        "Bypass".to_string(),
                        "-Command".to_string(),
                        expr,
                    ],
                };
            }
            return SpawnSpec {
                available: true,
                command: path,
                args: args.to_vec(),
            };
        }
        return SpawnSpec {
            available: false,
            command: command.to_string(),
            args: args.to_vec(),
        };
    }

    #[cfg(not(windows))]
    {
        SpawnSpec {
            available: command_exists_non_windows(command),
            command: command.to_string(),
            args: args.to_vec(),
        }
    }
}

fn resolve_command_in_path(command: &str) -> Option<String> {
    let path = Path::new(command);
    if path.components().count() > 1 || path.is_absolute() {
        return path.exists().then(|| command.to_string());
    }

    let path_var = std::env::var_os("PATH")?;
    let mut candidates = vec![command.to_string()];
    if path.extension().is_none() {
        candidates.extend(
            [".exe", ".cmd", ".bat", ".com", ".ps1"]
                .iter()
                .map(|ext| format!("{command}{ext}")),
        );
    }

    for dir in std::env::split_paths(&path_var) {
        for candidate in &candidates {
            let full = dir.join(candidate);
            if full.exists() {
                return Some(full.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn command_exists_non_windows(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 || path.is_absolute() {
        return path.exists();
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(command).exists()))
        .unwrap_or(false)
}

fn native_codex_command() -> Option<(String, Vec<String>)> {
    #[cfg(windows)]
    {
        let node_dir = std::path::PathBuf::from(r"C:\Program Files\nodejs");
        let node = node_dir.join("node.exe");
        let codex_js = node_dir.join("node_modules").join("@openai").join("codex").join("bin").join("codex.js");
        if node.exists() && codex_js.exists() {
            return Some((
                node.to_string_lossy().to_string(),
                vec![
                    codex_js.to_string_lossy().to_string(),
                    "exec".to_string(),
                    "--skip-git-repo-check".to_string(),
                    "--sandbox".to_string(),
                    "workspace-write".to_string(),
                    "--full-auto".to_string(),
                ],
            ));
        }
    }
    None
}
