use reqwest::header::CONTENT_TYPE;

use crate::cdn::aes_ecb::encrypt_aes_ecb;
use crate::cdn::cdn_url::{build_cdn_download_url, build_cdn_upload_url};
use crate::error::{Result, WechatError};

const MAX_RETRIES: usize = 3;

pub async fn upload_buffer_to_cdn(
    client: &reqwest::Client,
    cdn_base_url: &str,
    upload_param: &str,
    filekey: &str,
    plaintext: &[u8],
    aes_key: &[u8; 16],
) -> Result<String> {
    let ciphertext = encrypt_aes_ecb(plaintext, aes_key);
    let url = build_cdn_upload_url(cdn_base_url, upload_param, filekey);

    let mut last_error: Option<WechatError> = None;
    for _ in 0..MAX_RETRIES {
        let res = client
            .post(&url)
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(ciphertext.clone())
            .send()
            .await?;

        if res.status().is_client_error() {
            let body = res.text().await.unwrap_or_default();
            return Err(WechatError::Api(format!("cdn client error: {body}")));
        }
        if !res.status().is_success() {
            last_error = Some(WechatError::Api(format!("cdn server error: {}", res.status())));
            continue;
        }

        let header = res
            .headers()
            .get("x-encrypted-param")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        if let Some(v) = header {
            return Ok(v);
        }
        last_error = Some(WechatError::InvalidResponse(
            "cdn response missing x-encrypted-param".to_string(),
        ));
    }

    Err(last_error.unwrap_or_else(|| WechatError::Api("cdn upload failed".to_string())))
}

pub async fn download_cdn_bytes(
    client: &reqwest::Client,
    cdn_base_url: &str,
    encrypted_query_param: &str,
) -> Result<Vec<u8>> {
    let url = build_cdn_download_url(cdn_base_url, encrypted_query_param);
    let res = client.get(url).send().await?;
    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(WechatError::Api(format!("cdn download failed: {body}")));
    }
    Ok(res.bytes().await?.to_vec())
}
