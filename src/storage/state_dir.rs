use std::path::PathBuf;

pub fn resolve_state_dir() -> PathBuf {
    if let Ok(v) = std::env::var("OPENCLAW_STATE_DIR") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    if let Ok(v) = std::env::var("CLAWDBOT_STATE_DIR") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".openclaw")
}
