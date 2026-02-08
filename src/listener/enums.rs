#[derive(Debug, Clone, Copy, Default)]
pub enum ContentType {
    Json,
    FormData,
    Text,
    Binary,
    #[default]
    Unknown,
}

impl ContentType {
    pub fn from_header_value(value: &[u8]) -> Self {
        let s = match std::str::from_utf8(value) {
            Ok(s) => s.trim().to_lowercase(),
            Err(_) => return ContentType::Unknown,
        };
        if s.starts_with("application/json") {
            ContentType::Json
        } else if s.starts_with("application/x-www-form-urlencoded") || s.starts_with("multipart/form-data") {
            ContentType::FormData
        } else if s.starts_with("text/") {
            ContentType::Text
        } else if s.starts_with("application/octet-stream") {
            ContentType::Binary
        } else {
            ContentType::Unknown
        }
    }
}