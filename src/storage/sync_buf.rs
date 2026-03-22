use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::state_dir::resolve_state_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncBufData {
    get_updates_buf: String,
}

pub fn sync_buf_path(account_id: &str) -> PathBuf {
    resolve_state_dir()
        .join("openclaw-weixin")
        .join("accounts")
        .join(format!("{account_id}.sync.json"))
}

pub fn load_sync_buf(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let data: SyncBufData = serde_json::from_str(&raw).ok()?;
    Some(data.get_updates_buf)
}

pub fn save_sync_buf(path: &Path, value: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = SyncBufData {
        get_updates_buf: value.to_string(),
    };
    fs::write(path, serde_json::to_vec(&data).unwrap_or_default())
}
