use std::time::Duration;

use crate::agent::Agent;
use crate::api::client::WeixinApiClient;
use crate::auth::accounts::{normalize_account_id, register_account_id, resolve_account, save_account, AccountData, DEFAULT_BASE_URL};
use crate::auth::login_qr::{fetch_qr_code, wait_for_qr_login, DEFAULT_ILINK_BOT_TYPE};
use crate::error::{Result, WechatError};
use crate::monitor::{MonitorOptions, MonitorRunner};

#[derive(Debug, Clone)]
pub struct LoginOptions {
    pub base_url: Option<String>,
    pub timeout: Duration,
}

impl Default for LoginOptions {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout: Duration::from_secs(480),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StartOptions {
    pub account_id: Option<String>,
}

pub struct Bot;

impl Bot {
    pub async fn login(opts: LoginOptions) -> Result<String> {
        let base_url = opts.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let client = reqwest::Client::builder().build()?;

        let start = fetch_qr_code(&client, &base_url, DEFAULT_ILINK_BOT_TYPE).await?;
        println!("请用微信扫描二维码链接:\n{}\n", start.qrcode_url);

        let result = wait_for_qr_login(&client, &base_url, start, opts.timeout, None).await?;
        if !result.connected {
            return Err(WechatError::Api("登录超时或失败".to_string()));
        }

        let account_id_raw = result
            .account_id
            .ok_or_else(|| WechatError::InvalidResponse("missing account id".to_string()))?;
        let account_id = normalize_account_id(&account_id_raw);
        let token = result
            .bot_token
            .ok_or_else(|| WechatError::InvalidResponse("missing bot token".to_string()))?;

        save_account(
            &account_id,
            AccountData {
                token: Some(token),
                base_url: result.base_url.or(Some(base_url)),
                user_id: result.user_id,
                saved_at: None,
            },
        )?;
        register_account_id(&account_id)?;

        Ok(account_id)
    }

    pub async fn start<A: Agent>(agent: A, opts: StartOptions) -> Result<()> {
        let account = resolve_account(opts.account_id.as_deref())
            .ok_or_else(|| WechatError::Api("没有可用账号，请先 login".to_string()))?;

        let api = WeixinApiClient::new(account.base_url.clone(), account.token.clone())?;

        let mut runner = MonitorRunner::new();
        runner
            .run(
                &api,
                &agent,
                MonitorOptions {
                    account_id: account.account_id,
                    cdn_base_url: account.cdn_base_url,
                    ..Default::default()
                },
            )
            .await
    }
}
