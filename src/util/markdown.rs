pub fn markdown_to_plain_text(input: &str) -> String {
    let mut s = input.replace("\r\n", "\n");
    while let Some(start) = s.find("```") {
        let end_slice = &s[start + 3..];
        if let Some(end_rel) = end_slice.find("```") {
            let end = start + 3 + end_rel;
            let content = s[start + 3..end].trim().to_string();
            s.replace_range(start..end + 3, &content);
        } else {
            break;
        }
    }
    s = s
        .replace("**", "")
        .replace("__", "")
        .replace("~~", "")
        .replace('`', "");
    s
}
