use std::collections::HashMap;
use std::path::PathBuf;

use crate::agent::{Agent, ChatRequest, MediaInput, MediaKind};
use crate::api::client::WeixinApiClient;
use crate::api::types::{enums, MessageItem, WeixinMessage};
use crate::cdn::download::{download_and_decrypt, download_plain};
use crate::error::Result;

use super::inbound::{body_from_items, find_media_item};
use super::send::send_text;
use super::send_media::send_media_file;

#[derive(Debug, Clone)]
pub struct ProcessContext {
    pub account_id: String,
    pub cdn_base_url: String,
    pub temp_dir: PathBuf,
}

fn pick_encrypt_param_and_key(item: &MessageItem) -> Option<(String, Option<String>, &'static str)> {
    if item.item_type == Some(enums::ITEM_IMAGE) {
        let media = item.image_item.as_ref()?.media.as_ref()?;
        let param = media.encrypt_query_param.clone()?;
        let key = item
            .image_item
            .as_ref()
            .and_then(|i| i.aeskey.clone().map(|h| {
                let bytes = hex::decode(h).unwrap_or_default();
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
            }))
            .or_else(|| media.aes_key.clone());
        return Some((param, key, "image"));
    }
    if item.item_type == Some(enums::ITEM_VIDEO) {
        let media = item.video_item.as_ref()?.media.as_ref()?;
        return Some((media.encrypt_query_param.clone()?, media.aes_key.clone(), "video"));
    }
    if item.item_type == Some(enums::ITEM_FILE) {
        let media = item.file_item.as_ref()?.media.as_ref()?;
        return Some((media.encrypt_query_param.clone()?, media.aes_key.clone(), "file"));
    }
    if item.item_type == Some(enums::ITEM_VOICE) {
        let media = item.voice_item.as_ref()?.media.as_ref()?;
        return Some((media.encrypt_query_param.clone()?, media.aes_key.clone(), "audio"));
    }
    None
}

async fn save_temp_media(temp_dir: &PathBuf, ext: &str, bytes: &[u8]) -> Result<PathBuf> {
    tokio::fs::create_dir_all(temp_dir).await?;
    let file = format!("{}{}.{}", "inbound-", uuid::Uuid::new_v4(), ext);
    let path = temp_dir.join(file);
    tokio::fs::write(&path, bytes).await?;
    Ok(path)
}

pub async fn process_one_message<A: Agent>(
    api: &WeixinApiClient,
    agent: &A,
    msg: &WeixinMessage,
    context: &ProcessContext,
    context_token_store: &mut HashMap<String, String>,
    typing_ticket: Option<&str>,
) -> Result<()> {
    let from = msg.from_user_id.clone().unwrap_or_default();
    let context_token = msg.context_token.clone().unwrap_or_default();
    if !context_token.is_empty() {
        context_token_store.insert(from.clone(), context_token.clone());
    }

    let items = msg.item_list.clone().unwrap_or_default();
    let text_body = body_from_items(&items);

    if let Some(ticket) = typing_ticket {
        let _ = api
            .send_typing(crate::api::types::SendTypingReq {
                ilink_user_id: from.clone(),
                typing_ticket: ticket.to_string(),
                status: enums::TYPING,
            })
            .await;
    }

    let mut request = ChatRequest {
        conversation_id: from.clone(),
        text: text_body.clone(),
        media: None,
    };

    if let Some(item) = find_media_item(&items) {
        if let Some((param, key, kind)) = pick_encrypt_param_and_key(&item) {
            let media_bytes = if let Some(k) = key {
                download_and_decrypt(&api.client, &context.cdn_base_url, &param, &k).await?
            } else {
                download_plain(&api.client, &context.cdn_base_url, &param).await?
            };

            let (ext, media_kind, mime) = match kind {
                "image" => ("jpg", MediaKind::Image, "image/*"),
                "video" => ("mp4", MediaKind::Video, "video/mp4"),
                "audio" => ("silk", MediaKind::Audio, "audio/silk"),
                _ => ("bin", MediaKind::File, "application/octet-stream"),
            };

            let path = save_temp_media(&context.temp_dir, ext, &media_bytes).await?;
            request.media = Some(MediaInput {
                kind: media_kind,
                file_path: path.to_string_lossy().to_string(),
                mime_type: mime.to_string(),
                file_name: item.file_item.as_ref().and_then(|f| f.file_name.clone()),
            });
        }
    }

    if text_body.starts_with("/echo ") {
        let echo = text_body.trim_start_matches("/echo ");
        send_text(api, &from, &context_token, echo).await?;
        return Ok(());
    }

    let response = agent.chat(request).await?;
    if let Some(media) = response.media {
        let path = if media.url.starts_with("http://") || media.url.starts_with("https://") {
            let bytes = api.client.get(&media.url).send().await?.bytes().await?;
            let local = save_temp_media(&context.temp_dir, "bin", &bytes).await?;
            local
        } else {
            PathBuf::from(media.url)
        };
        send_media_file(
            api,
            &context.cdn_base_url,
            &from,
            &context_token,
            &path,
            response.text.as_deref(),
        )
        .await?;
    } else if let Some(text) = response.text {
        send_text(api, &from, &context_token, &text).await?;
    }

    if let Some(ticket) = typing_ticket {
        let _ = api
            .send_typing(crate::api::types::SendTypingReq {
                ilink_user_id: from,
                typing_ticket: ticket.to_string(),
                status: enums::TYPING_CANCEL,
            })
            .await;
    }

    Ok(())
}
