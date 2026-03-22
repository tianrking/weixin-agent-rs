use std::path::Path;

use crate::api::client::WeixinApiClient;
use crate::cdn::upload::{upload_file, upload_image, upload_video};
use crate::error::Result;
use crate::media::mime::guess_mime_from_path;

use super::send::{send_file, send_image, send_video};

pub async fn send_media_file(
    api: &WeixinApiClient,
    cdn_base_url: &str,
    to: &str,
    context_token: &str,
    file_path: &Path,
    text: Option<&str>,
) -> Result<()> {
    let mime = guess_mime_from_path(file_path);
    if mime.starts_with("image/") {
        let uploaded = upload_image(api, cdn_base_url, file_path, to).await?;
        return send_image(api, to, context_token, &uploaded, text).await;
    }

    if mime.starts_with("video/") {
        let uploaded = upload_video(api, cdn_base_url, file_path, to).await?;
        return send_video(api, to, context_token, &uploaded, text).await;
    }

    let uploaded = upload_file(api, cdn_base_url, file_path, to).await?;
    let file_name = file_path
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| "file.bin".to_string());
    send_file(api, to, context_token, &uploaded, &file_name, text).await
}
