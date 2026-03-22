use std::time::Duration;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::api::types::{BaseInfo, GetConfigResp, GetUpdatesReq, GetUpdatesResp, GetUploadUrlReq, GetUploadUrlResp, SendMessageReq, SendTypingReq};
use crate::error::{Result, WechatError};
use crate::util::random::random_wechat_uin_base64;

const DEFAULT_LONG_POLL_TIMEOUT: u64 = 35_000;
const DEFAULT_API_TIMEOUT: u64 = 15_000;
const DEFAULT_CONFIG_TIMEOUT: u64 = 10_000;

#[derive(Debug, Clone)]
pub struct WeixinApiClient {
    pub base_url: String,
    pub token: String,
    pub route_tag: Option<String>,
    pub client: reqwest::Client,
    pub channel_version: String,
}

impl WeixinApiClient {
    pub fn new(base_url: String, token: String) -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self {
            base_url,
            token,
            route_tag: None,
            client,
            channel_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    fn build_base_info(&self) -> BaseInfo {
        BaseInfo {
            channel_version: self.channel_version.clone(),
        }
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("AuthorizationType", HeaderValue::from_static("ilink_bot_token"));
        headers.insert(
            "X-WECHAT-UIN",
            HeaderValue::from_str(&random_wechat_uin_base64())
                .map_err(|e| WechatError::InvalidResponse(format!("invalid X-WECHAT-UIN: {e}")))?,
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .map_err(|e| WechatError::InvalidResponse(format!("invalid auth header: {e}")))?,
        );
        if let Some(tag) = &self.route_tag {
            headers.insert(
                "SKRouteTag",
                HeaderValue::from_str(tag)
                    .map_err(|e| WechatError::InvalidResponse(format!("invalid route tag: {e}")))?,
            );
        }
        Ok(headers)
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: serde_json::Value,
        timeout_ms: u64,
    ) -> Result<T> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint);
        let res = self
            .client
            .post(url)
            .headers(self.build_headers()?)
            .timeout(Duration::from_millis(timeout_ms))
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;
        if !status.is_success() {
            return Err(WechatError::Api(format!("{}: {text}", status.as_u16())));
        }
        Ok(serde_json::from_str(&text)?)
    }

    async fn post_empty(&self, endpoint: &str, body: serde_json::Value, timeout_ms: u64) -> Result<()> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint);
        let res = self
            .client
            .post(url)
            .headers(self.build_headers()?)
            .timeout(Duration::from_millis(timeout_ms))
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(WechatError::Api(format!("{}: {text}", status.as_u16())));
        }
        Ok(())
    }

    pub async fn get_updates(&self, req: GetUpdatesReq, timeout_ms: Option<u64>) -> Result<GetUpdatesResp> {
        let timeout = timeout_ms.unwrap_or(DEFAULT_LONG_POLL_TIMEOUT);
        let body = json!({
            "get_updates_buf": req.get_updates_buf,
            "base_info": self.build_base_info(),
        });
        self.post_json("ilink/bot/getupdates", body, timeout).await
    }

    pub async fn send_message(&self, req: SendMessageReq) -> Result<()> {
        self.post_empty(
            "ilink/bot/sendmessage",
            json!({ "msg": req.msg, "base_info": self.build_base_info() }),
            DEFAULT_API_TIMEOUT,
        )
        .await
    }

    pub async fn get_upload_url(&self, req: GetUploadUrlReq) -> Result<GetUploadUrlResp> {
        self.post_json(
            "ilink/bot/getuploadurl",
            json!({
                "filekey": req.filekey,
                "media_type": req.media_type,
                "to_user_id": req.to_user_id,
                "rawsize": req.rawsize,
                "rawfilemd5": req.rawfilemd5,
                "filesize": req.filesize,
                "thumb_rawsize": req.thumb_rawsize,
                "thumb_rawfilemd5": req.thumb_rawfilemd5,
                "thumb_filesize": req.thumb_filesize,
                "no_need_thumb": req.no_need_thumb,
                "aeskey": req.aeskey,
                "base_info": self.build_base_info(),
            }),
            DEFAULT_API_TIMEOUT,
        )
        .await
    }

    pub async fn get_config(&self, user_id: &str, context_token: Option<&str>) -> Result<GetConfigResp> {
        self.post_json(
            "ilink/bot/getconfig",
            json!({
                "ilink_user_id": user_id,
                "context_token": context_token,
                "base_info": self.build_base_info(),
            }),
            DEFAULT_CONFIG_TIMEOUT,
        )
        .await
    }

    pub async fn send_typing(&self, req: SendTypingReq) -> Result<()> {
        self.post_empty(
            "ilink/bot/sendtyping",
            json!({
                "ilink_user_id": req.ilink_user_id,
                "typing_ticket": req.typing_ticket,
                "status": req.status,
                "base_info": self.build_base_info(),
            }),
            DEFAULT_CONFIG_TIMEOUT,
        )
        .await
    }
}
