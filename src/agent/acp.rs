use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;

use crate::error::{Result, WechatError};

use super::{Agent, ChatRequest, ChatResponse, MediaOutKind, MediaOutput};

#[derive(Debug, Clone)]
pub struct AcpAgentOptions {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub prompt_timeout: Duration,
}

impl Default for AcpAgentOptions {
    fn default() -> Self {
        Self {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@zed-industries/codex-acp".to_string()],
            cwd: None,
            env: HashMap::new(),
            prompt_timeout: Duration::from_secs(120),
        }
    }
}

#[derive(Debug)]
struct PendingRequest {
    tx: oneshot::Sender<Result<Value>>,
}

#[derive(Debug)]
struct SessionCollector {
    text_chunks: String,
    image_data: Option<(String, String)>,
}

impl SessionCollector {
    fn new() -> Self {
        Self {
            text_chunks: String::new(),
            image_data: None,
        }
    }
}

struct Inner {
    stdin: Arc<Mutex<ChildStdin>>,
    request_id: u64,
    pending: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    collectors: Arc<Mutex<HashMap<String, SessionCollector>>>,
    session_by_conversation: HashMap<String, String>,
    prompt_timeout: Duration,
    _child: Child,
}

pub struct AcpAgent {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcEnvelope {
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<Value>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<Value>,
}

impl AcpAgent {
    pub async fn new(opts: AcpAgentOptions) -> Result<Self> {
        let mut cmd = Command::new(&opts.command);
        cmd.args(&opts.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        if let Some(cwd) = &opts.cwd {
            cmd.current_dir(cwd);
        }
        for (k, v) in &opts.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| WechatError::Api(format!("failed to spawn acp process: {e}")))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| WechatError::InvalidResponse("acp stdin unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| WechatError::InvalidResponse("acp stdout unavailable".to_string()))?;

        let pending = Arc::new(Mutex::new(HashMap::<u64, PendingRequest>::new()));
        let collectors = Arc::new(Mutex::new(HashMap::<String, SessionCollector>::new()));

        let read_pending = Arc::clone(&pending);
        let read_collectors = Arc::clone(&collectors);
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let parsed: Result<JsonRpcEnvelope> = serde_json::from_str::<JsonRpcEnvelope>(&line)
                    .map_err(WechatError::from);
                let envelope = match parsed {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(id) = envelope.id {
                    let tx = {
                        let mut p = read_pending.lock().await;
                        p.remove(&id).map(|v| v.tx)
                    };
                    if let Some(reply) = tx {
                        if let Some(err) = envelope.error {
                            let _ = reply.send(Err(WechatError::Api(format!("acp rpc error: {err}"))));
                        } else {
                            let _ = reply.send(Ok(envelope.result.unwrap_or(Value::Null)));
                        }
                    }
                    continue;
                }

                if envelope.method.as_deref() == Some("sessionUpdate") {
                    let params = envelope.params.unwrap_or(Value::Null);
                    let session_id = params
                        .get("sessionId")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let update = params.get("update").cloned().unwrap_or(Value::Null);
                    let update_type = update
                        .get("sessionUpdate")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    if update_type == "agent_message_chunk" {
                        let content = update.get("content").cloned().unwrap_or(Value::Null);
                        let kind = content.get("type").and_then(|v| v.as_str()).unwrap_or_default();
                        let mut locked = read_collectors.lock().await;
                        let collector = locked.entry(session_id).or_insert_with(SessionCollector::new);
                        match kind {
                            "text" => {
                                if let Some(t) = content.get("text").and_then(|v| v.as_str()) {
                                    collector.text_chunks.push_str(t);
                                }
                            }
                            "image" => {
                                let data = content.get("data").and_then(|v| v.as_str()).unwrap_or_default();
                                let mime = content.get("mimeType").and_then(|v| v.as_str()).unwrap_or("image/png");
                                collector.image_data = Some((data.to_string(), mime.to_string()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        let agent = Self {
            inner: Arc::new(Mutex::new(Inner {
                stdin: Arc::new(Mutex::new(stdin)),
                request_id: 1,
                pending,
                collectors,
                session_by_conversation: HashMap::new(),
                prompt_timeout: opts.prompt_timeout,
                _child: child,
            })),
        };

        agent
            .rpc_call(
                "initialize",
                json!({
                    "protocolVersion": "0.2",
                    "clientInfo": { "name": "wechat-rs-sdk", "version": env!("CARGO_PKG_VERSION") },
                    "clientCapabilities": {}
                }),
            )
            .await?;

        Ok(agent)
    }

    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value> {
        let (id, stdin, pending_map, tx, rx) = {
            let mut inner = self.inner.lock().await;
            let id = inner.request_id;
            inner.request_id += 1;
            let (tx, rx) = oneshot::channel();
            (id, Arc::clone(&inner.stdin), Arc::clone(&inner.pending), tx, rx)
        };

        {
            let mut pending = pending_map.lock().await;
            pending.insert(id, PendingRequest { tx });
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let mut writer = stdin.lock().await;
        writer
            .write_all(serde_json::to_string(&payload)?.as_bytes())
            .await
            .map_err(WechatError::from)?;
        writer.write_all(b"\n").await.map_err(WechatError::from)?;
        writer.flush().await.map_err(WechatError::from)?;

        rx.await.map_err(|_| WechatError::Api("acp rpc response channel closed".to_string()))?
    }

    async fn ensure_session(&self, conversation_id: &str) -> Result<String> {
        {
            let inner = self.inner.lock().await;
            if let Some(session) = inner.session_by_conversation.get(conversation_id) {
                return Ok(session.clone());
            }
        }

        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        let result = self
            .rpc_call(
                "newSession",
                json!({
                    "cwd": cwd,
                    "mcpServers": []
                }),
            )
            .await?;

        let session_id = result
            .get("sessionId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WechatError::InvalidResponse("acp newSession missing sessionId".to_string()))?
            .to_string();

        let mut inner = self.inner.lock().await;
        inner
            .session_by_conversation
            .insert(conversation_id.to_string(), session_id.clone());
        Ok(session_id)
    }

    async fn prompt(&self, session_id: &str, request: &ChatRequest) -> Result<ChatResponse> {
        {
            let collectors_arc = {
                let inner = self.inner.lock().await;
                Arc::clone(&inner.collectors)
            };
            let mut collectors = collectors_arc.lock().await;
            collectors.insert(session_id.to_string(), SessionCollector::new());
        }

        let mut blocks = Vec::new();
        if !request.text.trim().is_empty() {
            blocks.push(json!({ "type": "text", "text": request.text }));
        }

        if let Some(media) = &request.media {
            let bytes = tokio::fs::read(&media.file_path).await?;
            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
            match media.kind {
                super::MediaKind::Image => {
                    blocks.push(json!({ "type": "image", "data": b64, "mimeType": media.mime_type }));
                }
                super::MediaKind::Audio => {
                    blocks.push(json!({ "type": "audio", "data": b64, "mimeType": media.mime_type }));
                }
                _ => {
                    blocks.push(json!({
                        "type": "resource",
                        "resource": {
                            "uri": format!("file://{}", media.file_path),
                            "blob": b64,
                            "mimeType": media.mime_type,
                        }
                    }));
                }
            }
        }

        let wait = {
            let inner = self.inner.lock().await;
            inner.prompt_timeout
        };

        timeout(
            wait,
            self.rpc_call(
                "prompt",
                json!({
                    "sessionId": session_id,
                    "prompt": blocks,
                }),
            ),
        )
        .await
        .map_err(|_| WechatError::Api("acp prompt timeout".to_string()))??;

        tokio::time::sleep(Duration::from_millis(300)).await;

        let mut response = ChatResponse {
            text: None,
            media: None,
        };

        let collector = {
            let collectors_arc = {
                let inner = self.inner.lock().await;
                Arc::clone(&inner.collectors)
            };
            let mut collectors = collectors_arc.lock().await;
            collectors.remove(session_id)
        };

        if let Some(c) = collector {
            if !c.text_chunks.is_empty() {
                response.text = Some(c.text_chunks);
            }
            if let Some((img_b64, mime)) = c.image_data {
                let out_dir = std::env::temp_dir().join("wechat-rs-sdk").join("acp-out");
                tokio::fs::create_dir_all(&out_dir).await?;
                let ext = mime.split('/').nth(1).unwrap_or("png");
                let out = out_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
                let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, img_b64)
                    .map_err(|e| WechatError::InvalidResponse(format!("invalid acp image base64: {e}")))?;
                tokio::fs::write(&out, bytes).await?;
                response.media = Some(MediaOutput {
                    kind: MediaOutKind::Image,
                    url: out.to_string_lossy().to_string(),
                    file_name: None,
                });
            }
        }

        Ok(response)
    }
}

#[async_trait]
impl Agent for AcpAgent {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let session_id = self.ensure_session(&request.conversation_id).await?;
        self.prompt(&session_id, &request).await
    }
}
