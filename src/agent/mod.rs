use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub conversation_id: String,
    pub text: String,
    pub media: Option<MediaInput>,
}

#[derive(Debug, Clone)]
pub struct MediaInput {
    pub kind: MediaKind,
    pub file_path: String,
    pub mime_type: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: Option<String>,
    pub media: Option<MediaOutput>,
}

#[derive(Debug, Clone)]
pub struct MediaOutput {
    pub kind: MediaOutKind,
    pub url: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MediaKind {
    Image,
    Audio,
    Video,
    File,
}

#[derive(Debug, Clone, Copy)]
pub enum MediaOutKind {
    Image,
    Video,
    File,
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}
