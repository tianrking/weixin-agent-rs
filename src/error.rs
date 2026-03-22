use thiserror::Error;

pub type Result<T> = std::result::Result<T, WechatError>;

#[derive(Debug, Error)]
pub enum WechatError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("api error: {0}")]
    Api(String),
    #[error("session paused: {0}")]
    SessionPaused(String),
}
