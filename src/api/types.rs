use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaseInfo {
    pub channel_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetUpdatesReq {
    #[serde(default)]
    pub get_updates_buf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetUpdatesResp {
    pub ret: Option<i32>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub msgs: Option<Vec<WeixinMessage>>,
    pub get_updates_buf: Option<String>,
    pub longpolling_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SendMessageReq {
    pub msg: WeixinMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetUploadUrlReq {
    pub filekey: String,
    pub media_type: i32,
    pub to_user_id: String,
    pub rawsize: u64,
    pub rawfilemd5: String,
    pub filesize: u64,
    pub thumb_rawsize: Option<u64>,
    pub thumb_rawfilemd5: Option<String>,
    pub thumb_filesize: Option<u64>,
    pub no_need_thumb: Option<bool>,
    pub aeskey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetUploadUrlResp {
    pub upload_param: Option<String>,
    pub thumb_upload_param: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SendTypingReq {
    pub ilink_user_id: String,
    pub typing_ticket: String,
    pub status: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetConfigResp {
    pub ret: Option<i32>,
    pub errmsg: Option<String>,
    pub typing_ticket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeixinMessage {
    pub seq: Option<u64>,
    pub message_id: Option<u64>,
    pub from_user_id: Option<String>,
    pub to_user_id: Option<String>,
    pub client_id: Option<String>,
    pub create_time_ms: Option<u64>,
    pub session_id: Option<String>,
    pub message_type: Option<i32>,
    pub message_state: Option<i32>,
    pub item_list: Option<Vec<MessageItem>>,
    pub context_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageItem {
    #[serde(rename = "type")]
    pub item_type: Option<i32>,
    pub text_item: Option<TextItem>,
    pub image_item: Option<ImageItem>,
    pub voice_item: Option<VoiceItem>,
    pub file_item: Option<FileItem>,
    pub video_item: Option<VideoItem>,
    pub ref_msg: Option<RefMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextItem {
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CdnMedia {
    pub encrypt_query_param: Option<String>,
    pub aes_key: Option<String>,
    pub encrypt_type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageItem {
    pub media: Option<CdnMedia>,
    pub thumb_media: Option<CdnMedia>,
    pub aeskey: Option<String>,
    pub mid_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceItem {
    pub media: Option<CdnMedia>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileItem {
    pub media: Option<CdnMedia>,
    pub file_name: Option<String>,
    pub len: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoItem {
    pub media: Option<CdnMedia>,
    pub video_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefMessage {
    pub message_item: Option<Box<MessageItem>>,
    pub title: Option<String>,
}

pub mod enums {
    pub const UPLOAD_MEDIA_IMAGE: i32 = 1;
    pub const UPLOAD_MEDIA_VIDEO: i32 = 2;
    pub const UPLOAD_MEDIA_FILE: i32 = 3;

    pub const MESSAGE_TYPE_BOT: i32 = 2;

    pub const MESSAGE_STATE_FINISH: i32 = 2;

    pub const ITEM_TEXT: i32 = 1;
    pub const ITEM_IMAGE: i32 = 2;
    pub const ITEM_VOICE: i32 = 3;
    pub const ITEM_FILE: i32 = 4;
    pub const ITEM_VIDEO: i32 = 5;

    pub const TYPING: i32 = 1;
    pub const TYPING_CANCEL: i32 = 2;
}
