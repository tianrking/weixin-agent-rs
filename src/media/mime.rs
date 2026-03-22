use std::path::Path;

pub fn guess_mime_from_path(path: &Path) -> String {
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}
