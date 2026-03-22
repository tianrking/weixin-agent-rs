use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::api::client::WeixinApiClient;
use crate::error::Result;

#[derive(Debug, Clone, Default)]
pub struct CachedConfig {
    pub typing_ticket: String,
}

#[derive(Debug, Clone)]
struct Entry {
    config: CachedConfig,
    next_fetch_at: Instant,
    retry_delay: Duration,
}

#[derive(Debug, Default)]
pub struct WeixinConfigManager {
    cache: HashMap<String, Entry>,
}

impl WeixinConfigManager {
    pub async fn get_for_user(
        &mut self,
        api: &WeixinApiClient,
        user_id: &str,
        context_token: Option<&str>,
    ) -> Result<CachedConfig> {
        let now = Instant::now();
        let should_fetch = self
            .cache
            .get(user_id)
            .map(|e| now >= e.next_fetch_at)
            .unwrap_or(true);

        if should_fetch {
            match api.get_config(user_id, context_token).await {
                Ok(resp) if resp.ret.unwrap_or(0) == 0 => {
                    self.cache.insert(
                        user_id.to_string(),
                        Entry {
                            config: CachedConfig {
                                typing_ticket: resp.typing_ticket.unwrap_or_default(),
                            },
                            next_fetch_at: now + Duration::from_secs(60 * 60),
                            retry_delay: Duration::from_secs(2),
                        },
                    );
                }
                _ => {
                    let prev = self.cache.get(user_id).map(|e| e.retry_delay).unwrap_or(Duration::from_secs(2));
                    let next = (prev * 2).min(Duration::from_secs(60 * 60));
                    self.cache.entry(user_id.to_string()).and_modify(|e| {
                        e.next_fetch_at = now + next;
                        e.retry_delay = next;
                    }).or_insert(Entry {
                        config: CachedConfig::default(),
                        next_fetch_at: now + Duration::from_secs(2),
                        retry_delay: Duration::from_secs(2),
                    });
                }
            }
        }

        Ok(self
            .cache
            .get(user_id)
            .map(|e| e.config.clone())
            .unwrap_or_default())
    }
}
