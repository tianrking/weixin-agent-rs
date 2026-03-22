use crate::cdn::aes_ecb::decrypt_aes_ecb;
use crate::cdn::cdn_transfer::download_cdn_bytes;
use crate::error::{Result, WechatError};

pub fn parse_aes_key_b64(aes_key_b64: &str) -> Result<[u8; 16]> {
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, aes_key_b64)
        .map_err(|e| WechatError::InvalidResponse(format!("invalid aes key b64: {e}")))?;

    if decoded.len() == 16 {
        let mut key = [0u8; 16];
        key.copy_from_slice(&decoded);
        return Ok(key);
    }

    if decoded.len() == 32 {
        let text = std::str::from_utf8(&decoded)
            .map_err(|e| WechatError::InvalidResponse(format!("invalid aes hex bytes: {e}")))?;
        let bytes = hex::decode(text)
            .map_err(|e| WechatError::InvalidResponse(format!("invalid aes hex: {e}")))?;
        if bytes.len() == 16 {
            let mut key = [0u8; 16];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
    }

    Err(WechatError::InvalidResponse(
        "aes key must decode to 16 bytes or 32-char hex".to_string(),
    ))
}

pub async fn download_and_decrypt(
    client: &reqwest::Client,
    cdn_base_url: &str,
    encrypted_query_param: &str,
    aes_key_b64: &str,
) -> Result<Vec<u8>> {
    let key = parse_aes_key_b64(aes_key_b64)?;
    let encrypted = download_cdn_bytes(client, cdn_base_url, encrypted_query_param).await?;
    decrypt_aes_ecb(&encrypted, &key)
}

pub async fn download_plain(
    client: &reqwest::Client,
    cdn_base_url: &str,
    encrypted_query_param: &str,
) -> Result<Vec<u8>> {
    download_cdn_bytes(client, cdn_base_url, encrypted_query_param).await
}
