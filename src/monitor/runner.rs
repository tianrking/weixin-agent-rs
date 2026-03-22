use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use tracing::{error, info, warn};

use crate::agent::Agent;
use crate::api::client::WeixinApiClient;
use crate::api::config_cache::WeixinConfigManager;
use crate::api::types::GetUpdatesReq;
use crate::error::Result;
use crate::messaging::process::{process_one_message, ProcessContext};
use crate::storage::sync_buf::{load_sync_buf, save_sync_buf, sync_buf_path};

const SESSION_EXPIRED_ERRCODE: i32 = -14;

#[derive(Debug, Clone)]
pub struct MonitorOptions {
    pub account_id: String,
    pub cdn_base_url: String,
    pub long_poll_timeout_ms: u64,
    pub temp_dir: PathBuf,
}

impl Default for MonitorOptions {
    fn default() -> Self {
        Self {
            account_id: String::new(),
            cdn_base_url: "https://novac2c.cdn.weixin.qq.com/c2c".to_string(),
            long_poll_timeout_ms: 35_000,
            temp_dir: std::env::temp_dir().join("wechat-rs-sdk").join("media"),
        }
    }
}

pub struct MonitorRunner {
    pause_until: Option<std::time::Instant>,
    context_token_store: HashMap<String, String>,
    config_manager: WeixinConfigManager,
}

impl MonitorRunner {
    pub fn new() -> Self {
        Self {
            pause_until: None,
            context_token_store: HashMap::new(),
            config_manager: WeixinConfigManager::default(),
        }
    }

    pub async fn run<A: Agent>(
        &mut self,
        api: &WeixinApiClient,
        agent: &A,
        opts: MonitorOptions,
    ) -> Result<()> {
        let sync_path = sync_buf_path(&opts.account_id);
        let mut sync_buf = load_sync_buf(&sync_path).unwrap_or_default();
        let mut next_timeout = opts.long_poll_timeout_ms;

        let process_ctx = ProcessContext {
            account_id: opts.account_id.clone(),
            cdn_base_url: opts.cdn_base_url,
            temp_dir: opts.temp_dir,
        };

        loop {
            if let Some(until) = self.pause_until {
                if std::time::Instant::now() < until {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                self.pause_until = None;
            }

            let resp = api
                .get_updates(
                    GetUpdatesReq {
                        get_updates_buf: sync_buf.clone(),
                    },
                    Some(next_timeout),
                )
                .await;

            let resp = match resp {
                Ok(v) => v,
                Err(err) => {
                    warn!("getupdates error: {err}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            if let Some(t) = resp.longpolling_timeout_ms {
                if t > 0 {
                    next_timeout = t;
                }
            }

            if resp.ret.unwrap_or(0) != 0 || resp.errcode.unwrap_or(0) != 0 {
                let code = resp.errcode.unwrap_or(resp.ret.unwrap_or(0));
                if code == SESSION_EXPIRED_ERRCODE {
                    return Err(crate::error::WechatError::Api(
                        "session expired (errcode -14), please run login again".to_string(),
                    ));
                } else {
                    warn!(
                        "getupdates api error ret={:?} errcode={:?} msg={:?}",
                        resp.ret, resp.errcode, resp.errmsg
                    );
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                continue;
            }

            if let Some(next_buf) = resp.get_updates_buf.clone() {
                if !next_buf.is_empty() {
                    sync_buf = next_buf;
                    if let Err(err) = save_sync_buf(&sync_path, &sync_buf) {
                        warn!("save sync buf failed: {err}");
                    }
                }
            }

            let message_count = resp.msgs.as_ref().map(|v| v.len()).unwrap_or(0);
            let messages = resp.msgs.unwrap_or_default();
            for msg in messages {
                let user = msg.from_user_id.clone().unwrap_or_default();
                let cfg = self
                    .config_manager
                    .get_for_user(api, &user, msg.context_token.as_deref())
                    .await
                    .unwrap_or_default();

                if let Err(err) = process_one_message(
                    api,
                    agent,
                    &msg,
                    &process_ctx,
                    &mut self.context_token_store,
                    Some(cfg.typing_ticket.as_str()).filter(|v| !v.is_empty()),
                )
                .await
                {
                    error!("process message failed: {err}");
                }
            }

            info!("poll cycle done, messages={message_count}");
        }
    }
}
