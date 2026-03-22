use std::path::Path;

use crate::api::client::WeixinApiClient;
use crate::api::types::{enums, GetUploadUrlReq};
use crate::cdn::aes_ecb::aes_ecb_padded_size;
use crate::cdn::cdn_transfer::upload_buffer_to_cdn;
use crate::error::{Result, WechatError};
use crate::util::random::{random_bytes_16, random_hex_16};

#[derive(Debug, Clone)]
pub struct UploadedFileInfo {
    pub filekey: String,
    pub download_encrypted_query_param: String,
    pub aes_key_hex: String,
    pub file_size: u64,
    pub file_size_ciphertext: u64,
}

async fn upload_media(
    api: &WeixinApiClient,
    cdn_base_url: &str,
    file_path: &Path,
    to_user_id: &str,
    media_type: i32,
) -> Result<UploadedFileInfo> {
    let plaintext = tokio::fs::read(file_path).await?;
    let raw_size = plaintext.len() as u64;
    let raw_md5 = format!("{:x}", md5::compute(&plaintext));
    let cipher_size = aes_ecb_padded_size(plaintext.len()) as u64;

    let filekey = random_hex_16();
    let aes_key = random_bytes_16();
    let aes_key_hex = hex::encode(aes_key);

    let resp = api
        .get_upload_url(GetUploadUrlReq {
            filekey: filekey.clone(),
            media_type,
            to_user_id: to_user_id.to_string(),
            rawsize: raw_size,
            rawfilemd5: raw_md5,
            filesize: cipher_size,
            thumb_rawsize: None,
            thumb_rawfilemd5: None,
            thumb_filesize: None,
            no_need_thumb: Some(true),
            aeskey: Some(aes_key_hex.clone()),
        })
        .await?;

    let upload_param = resp
        .upload_param
        .ok_or_else(|| WechatError::InvalidResponse("missing upload_param".to_string()))?;

    let encrypted_param = upload_buffer_to_cdn(
        &api.client,
        cdn_base_url,
        &upload_param,
        &filekey,
        &plaintext,
        &aes_key,
    )
    .await?;

    Ok(UploadedFileInfo {
        filekey,
        download_encrypted_query_param: encrypted_param,
        aes_key_hex,
        file_size: raw_size,
        file_size_ciphertext: cipher_size,
    })
}

pub async fn upload_image(
    api: &WeixinApiClient,
    cdn_base_url: &str,
    file_path: &Path,
    to_user_id: &str,
) -> Result<UploadedFileInfo> {
    upload_media(api, cdn_base_url, file_path, to_user_id, enums::UPLOAD_MEDIA_IMAGE).await
}

pub async fn upload_video(
    api: &WeixinApiClient,
    cdn_base_url: &str,
    file_path: &Path,
    to_user_id: &str,
) -> Result<UploadedFileInfo> {
    upload_media(api, cdn_base_url, file_path, to_user_id, enums::UPLOAD_MEDIA_VIDEO).await
}

pub async fn upload_file(
    api: &WeixinApiClient,
    cdn_base_url: &str,
    file_path: &Path,
    to_user_id: &str,
) -> Result<UploadedFileInfo> {
    upload_media(api, cdn_base_url, file_path, to_user_id, enums::UPLOAD_MEDIA_FILE).await
}
