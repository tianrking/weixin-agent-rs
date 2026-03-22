use crate::api::types::{enums, MessageItem};

pub fn body_from_items(items: &[MessageItem]) -> String {
    for item in items {
        if item.item_type == Some(enums::ITEM_TEXT) {
            if let Some(text) = item.text_item.as_ref().and_then(|t| t.text.clone()) {
                if let Some(ref_msg) = &item.ref_msg {
                    if let Some(title) = &ref_msg.title {
                        return format!("[引用: {title}]\n{text}");
                    }
                }
                return text;
            }
        }
        if item.item_type == Some(enums::ITEM_VOICE) {
            if let Some(text) = item.voice_item.as_ref().and_then(|v| v.text.clone()) {
                return text;
            }
        }
    }
    String::new()
}

pub fn find_media_item(items: &[MessageItem]) -> Option<MessageItem> {
    items
        .iter()
        .find(|i| i.item_type == Some(enums::ITEM_IMAGE))
        .or_else(|| items.iter().find(|i| i.item_type == Some(enums::ITEM_VIDEO)))
        .or_else(|| items.iter().find(|i| i.item_type == Some(enums::ITEM_FILE)))
        .or_else(|| items.iter().find(|i| i.item_type == Some(enums::ITEM_VOICE)))
        .cloned()
}
