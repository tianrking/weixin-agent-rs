pub fn build_cdn_download_url(cdn_base_url: &str, encrypted_query_param: &str) -> String {
    format!(
        "{}/download?encrypted_query_param={}",
        cdn_base_url.trim_end_matches('/'),
        urlencoding::encode(encrypted_query_param)
    )
}

pub fn build_cdn_upload_url(cdn_base_url: &str, upload_param: &str, filekey: &str) -> String {
    format!(
        "{}/upload?encrypted_query_param={}&filekey={}",
        cdn_base_url.trim_end_matches('/'),
        urlencoding::encode(upload_param),
        urlencoding::encode(filekey)
    )
}
