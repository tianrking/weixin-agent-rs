use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::storage::state_dir::resolve_state_dir;

pub const DEFAULT_BASE_URL: &str = "https://ilinkai.weixin.qq.com";
pub const CDN_BASE_URL: &str = "https://novac2c.cdn.weixin.qq.com/c2c";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountData {
    pub token: Option<String>,
    pub base_url: Option<String>,
    pub user_id: Option<String>,
    pub saved_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedAccount {
    pub account_id: String,
    pub base_url: String,
    pub cdn_base_url: String,
    pub token: String,
}

fn weixin_dir() -> PathBuf {
    resolve_state_dir().join("openclaw-weixin")
}

fn accounts_dir() -> PathBuf {
    weixin_dir().join("accounts")
}

fn account_path(account_id: &str) -> PathBuf {
    accounts_dir().join(format!("{account_id}.json"))
}

fn index_path() -> PathBuf {
    weixin_dir().join("accounts.json")
}

pub fn normalize_account_id(raw: &str) -> String {
    raw.trim().to_lowercase().replace(['@', '.'], "-")
}

pub fn register_account_id(account_id: &str) -> std::io::Result<()> {
    fs::create_dir_all(weixin_dir())?;
    let mut ids = list_account_ids();
    if !ids.iter().any(|v| v == account_id) {
        ids.push(account_id.to_string());
    }
    fs::write(index_path(), serde_json::to_vec_pretty(&ids).unwrap_or_default())
}

pub fn list_account_ids() -> Vec<String> {
    let raw = match fs::read_to_string(index_path()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default()
}

pub fn load_account(account_id: &str) -> Option<AccountData> {
    let raw = fs::read_to_string(account_path(account_id)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_account(account_id: &str, mut data: AccountData) -> std::io::Result<()> {
    fs::create_dir_all(accounts_dir())?;
    data.saved_at = Some(format!("{}", chrono_like_timestamp()));
    fs::write(
        account_path(account_id),
        serde_json::to_vec_pretty(&data).unwrap_or_default(),
    )
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    ts.to_string()
}

pub fn resolve_account(account_id: Option<&str>) -> Option<ResolvedAccount> {
    let selected = if let Some(id) = account_id {
        normalize_account_id(id)
    } else {
        list_account_ids().first().cloned()?
    };

    let data = load_account(&selected)?;
    let token = data.token?.trim().to_string();
    if token.is_empty() {
        return None;
    }

    Some(ResolvedAccount {
        account_id: selected,
        base_url: data
            .base_url
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        cdn_base_url: CDN_BASE_URL.to_string(),
        token,
    })
}
