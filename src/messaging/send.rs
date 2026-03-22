use crate::api::client::WeixinApiClient;
use crate::api::types::{enums, CdnMedia, FileItem, ImageItem, MessageItem, SendMessageReq, TextItem, VideoItem, WeixinMessage};
use crate::cdn::upload::UploadedFileInfo;
use crate::error::{Result, WechatError};
use crate::util::markdown::markdown_to_plain_text;
use crate::util::random::generate_client_id;

fn base_msg(to: &str, context_token: &str) -> WeixinMessage {
    WeixinMessage {
        from_user_id: Some(String::new()),
        to_user_id: Some(to.to_string()),
        client_id: Some(generate_client_id("wechat-rs-sdk")),
        message_type: Some(enums::MESSAGE_TYPE_BOT),
        message_state: Some(enums::MESSAGE_STATE_FINISH),
        context_token: Some(context_token.to_string()),
        ..Default::default()
    }
}

pub async fn send_text(
    api: &WeixinApiClient,
    to: &str,
    context_token: &str,
    text: &str,
) -> Result<()> {
    if context_token.is_empty() {
        return Err(WechatError::InvalidResponse("context_token is required".to_string()));
    }

    let mut msg = base_msg(to, context_token);
    let plain = markdown_to_plain_text(text);
    msg.item_list = Some(vec![MessageItem {
        item_type: Some(enums::ITEM_TEXT),
        text_item: Some(TextItem { text: Some(plain) }),
        ..Default::default()
    }]);

    api.send_message(SendMessageReq { msg }).await
}

pub async fn send_image(
    api: &WeixinApiClient,
    to: &str,
    context_token: &str,
    uploaded: &UploadedFileInfo,
    text: Option<&str>,
) -> Result<()> {
    if let Some(t) = text {
        if !t.trim().is_empty() {
            send_text(api, to, context_token, t).await?;
        }
    }

    let mut msg = base_msg(to, context_token);
    msg.item_list = Some(vec![MessageItem {
        item_type: Some(enums::ITEM_IMAGE),
        image_item: Some(ImageItem {
            media: Some(CdnMedia {
                encrypt_query_param: Some(uploaded.download_encrypted_query_param.clone()),
                aes_key: Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    hex::decode(&uploaded.aes_key_hex).unwrap_or_default(),
                )),
                encrypt_type: Some(1),
            }),
            mid_size: Some(uploaded.file_size_ciphertext),
            ..Default::default()
        }),
        ..Default::default()
    }]);

    api.send_message(SendMessageReq { msg }).await
}

pub async fn send_video(
    api: &WeixinApiClient,
    to: &str,
    context_token: &str,
    uploaded: &UploadedFileInfo,
    text: Option<&str>,
) -> Result<()> {
    if let Some(t) = text {
        if !t.trim().is_empty() {
            send_text(api, to, context_token, t).await?;
        }
    }

    let mut msg = base_msg(to, context_token);
    msg.item_list = Some(vec![MessageItem {
        item_type: Some(enums::ITEM_VIDEO),
        video_item: Some(VideoItem {
            media: Some(CdnMedia {
                encrypt_query_param: Some(uploaded.download_encrypted_query_param.clone()),
                aes_key: Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    hex::decode(&uploaded.aes_key_hex).unwrap_or_default(),
                )),
                encrypt_type: Some(1),
            }),
            video_size: Some(uploaded.file_size_ciphertext),
        }),
        ..Default::default()
    }]);

    api.send_message(SendMessageReq { msg }).await
}

pub async fn send_file(
    api: &WeixinApiClient,
    to: &str,
    context_token: &str,
    uploaded: &UploadedFileInfo,
    file_name: &str,
    text: Option<&str>,
) -> Result<()> {
    if let Some(t) = text {
        if !t.trim().is_empty() {
            send_text(api, to, context_token, t).await?;
        }
    }

    let mut msg = base_msg(to, context_token);
    msg.item_list = Some(vec![MessageItem {
        item_type: Some(enums::ITEM_FILE),
        file_item: Some(FileItem {
            media: Some(CdnMedia {
                encrypt_query_param: Some(uploaded.download_encrypted_query_param.clone()),
                aes_key: Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    hex::decode(&uploaded.aes_key_hex).unwrap_or_default(),
                )),
                encrypt_type: Some(1),
            }),
            file_name: Some(file_name.to_string()),
            len: Some(uploaded.file_size.to_string()),
        }),
        ..Default::default()
    }]);

    api.send_message(SendMessageReq { msg }).await
}
