use rand::Rng;
use uuid::Uuid;

pub fn random_wechat_uin_base64() -> String {
    let value: u32 = rand::rng().random();
    let text = value.to_string();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, text.as_bytes())
}

pub fn generate_client_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

pub fn random_hex_16() -> String {
    let bytes: [u8; 16] = rand::rng().random();
    hex::encode(bytes)
}

pub fn random_bytes_16() -> [u8; 16] {
    rand::rng().random()
}
