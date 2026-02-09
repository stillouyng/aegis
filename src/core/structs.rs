use super::enums::ContentType;

#[derive(Debug, Default, Clone)]
pub struct RequestMeta {
    pub content_length: Option<usize>,
    pub content_type: Option<ContentType>,
    pub is_chunked: bool,
}

impl RequestMeta {
    pub fn new() -> Self {
        Self::default()
    }
}
