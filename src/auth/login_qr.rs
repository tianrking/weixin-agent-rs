use std::time::{Duration, Instant};

use reqwest::header::HeaderMap;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::error::{Result, WechatError};

pub const DEFAULT_ILINK_BOT_TYPE: &str = "3";
const POLL_TIMEOUT_MS: u64 = 35_000;
const MAX_QR_REFRESH_COUNT: u8 = 3;

#[derive(Debug, Clone)]
pub struct StartQrLogin {
    pub qrcode: String,
    pub qrcode_url: String,
}

#[derive(Debug, Clone)]
pub struct WaitQrLogin {
    pub connected: bool,
    pub bot_token: Option<String>,
    pub account_id: Option<String>,
    pub base_url: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QrCodeResponse {
    qrcode: String,
    qrcode_img_content: String,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    status: String,
    bot_token: Option<String>,
    ilink_bot_id: Option<String>,
    baseurl: Option<String>,
    ilink_user_id: Option<String>,
}

pub async fn fetch_qr_code(
    client: &reqwest::Client,
    base_url: &str,
    bot_type: &str,
) -> Result<StartQrLogin> {
    let base = format!("{}/", base_url.trim_end_matches('/'));
    let url = format!("{base}ilink/bot/get_bot_qrcode?bot_type={}", urlencoding::encode(bot_type));
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(WechatError::Api(format!("fetch qr failed: {}", resp.status())));
    }
    let parsed: QrCodeResponse = resp.json().await?;
    Ok(StartQrLogin {
        qrcode: parsed.qrcode,
        qrcode_url: parsed.qrcode_img_content,
    })
}

async fn poll_qr_status(
    client: &reqwest::Client,
    base_url: &str,
    qrcode: &str,
    mut headers: HeaderMap,
) -> Result<StatusResponse> {
    headers.insert("iLink-App-ClientVersion", "1".parse().unwrap());
    let base = format!("{}/", base_url.trim_end_matches('/'));
    let url = format!(
        "{base}ilink/bot/get_qrcode_status?qrcode={}",
        urlencoding::encode(qrcode)
    );

    let req = client
        .get(url)
        .headers(headers)
        .timeout(Duration::from_millis(POLL_TIMEOUT_MS));
    let resp = match req.send().await {
        Ok(v) => v,
        Err(err) if err.is_timeout() => {
            return Ok(StatusResponse {
                status: "wait".to_string(),
                bot_token: None,
                ilink_bot_id: None,
                baseurl: None,
                ilink_user_id: None,
            });
        }
        Err(err) => return Err(err.into()),
    };
    if !resp.status().is_success() {
        return Err(WechatError::Api(format!("poll qr failed: {}", resp.status())));
    }
    Ok(resp.json().await?)
}

pub async fn wait_for_qr_login(
    client: &reqwest::Client,
    base_url: &str,
    initial_qr: StartQrLogin,
    timeout: Duration,
    route_tag: Option<&str>,
) -> Result<WaitQrLogin> {
    let started = Instant::now();
    let mut qrcode = initial_qr.qrcode;
    let mut refreshes = 1u8;

    loop {
        if started.elapsed() > timeout {
            return Ok(WaitQrLogin {
                connected: false,
                bot_token: None,
                account_id: None,
                base_url: None,
                user_id: None,
            });
        }

        let mut headers = HeaderMap::new();
        if let Some(v) = route_tag {
            if !v.is_empty() {
                headers.insert("SKRouteTag", v.parse().unwrap());
            }
        }

        match poll_qr_status(client, base_url, &qrcode, headers).await {
            Ok(status) => match status.status.as_str() {
                "wait" => {}
                "scaned" => info!("二维码已扫码，等待确认"),
                "confirmed" => {
                    return Ok(WaitQrLogin {
                        connected: true,
                        bot_token: status.bot_token,
                        account_id: status.ilink_bot_id,
                        base_url: status.baseurl,
                        user_id: status.ilink_user_id,
                    })
                }
                "expired" => {
                    if refreshes >= MAX_QR_REFRESH_COUNT {
                        return Ok(WaitQrLogin {
                            connected: false,
                            bot_token: None,
                            account_id: None,
                            base_url: None,
                            user_id: None,
                        });
                    }
                    refreshes += 1;
                    warn!("二维码过期，刷新中 ({refreshes}/{MAX_QR_REFRESH_COUNT})");
                    let refreshed = fetch_qr_code(client, base_url, DEFAULT_ILINK_BOT_TYPE).await?;
                    qrcode = refreshed.qrcode;
                    println!("新二维码链接: {}", refreshed.qrcode_url);
                }
                other => debug!("未知扫码状态: {other}"),
            },
            Err(err) => debug!("轮询扫码状态失败(将重试): {err}"),
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
