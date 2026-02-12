#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ContentType {
    Json,
    FormData,
    Text,
    Binary,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    PayloadTooLarge = 413,
    InternalServerError = 500,
    NotImplemented = 501,
}

impl ContentType {
    pub fn from_header_value(value: &[u8]) -> Self {
        let s = match std::str::from_utf8(value) {
            Ok(s) => s.trim().to_lowercase(),
            Err(_) => return ContentType::Unknown,
        };
        if s.starts_with("application/json") {
            ContentType::Json
        } else if s.starts_with("application/x-www-form-urlencoded")
            || s.starts_with("multipart/form-data")
        {
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

impl HttpStatus {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            Self::Ok => b"200",
            Self::Created => b"201",
            Self::BadRequest => b"400",
            Self::NotFound => b"404",
            Self::PayloadTooLarge => b"413",
            _ => b"500",
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::Accepted => "Accepted",
            Self::NoContent => "No Content",
            Self::BadRequest => "Bad Request",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::PayloadTooLarge => "Payload Too Large",
            Self::InternalServerError => "Internal Server Error",
            Self::NotImplemented => "Not Implemented",
        }
    }
}

#[cfg(test)]
mod tests_contenttype {
    use super::*;

    #[test]
    fn test_content_type_parsing() {
        let cases = vec![
            (b"application/json" as &[u8], ContentType::Json),
            (b"application/json; charset=utf-8", ContentType::Json),
            (b"APPLICATION/JSON", ContentType::Json),
            (b"text/plain", ContentType::Text),
            (b"text/html; charset=ISO-8859-1", ContentType::Text),
            (b"application/x-www-form-urlencoded", ContentType::FormData),
            (
                b"multipart/form-data; boundary=something",
                ContentType::FormData,
            ),
            (b"application/octet-stream", ContentType::Binary),
            (b"unknown/type", ContentType::Unknown),
            (b"", ContentType::Unknown),
        ];

        for (input, _expected) in cases {
            let result = ContentType::from_header_value(input);
            assert!(
                matches!(result, _expected),
                "Failed on {:?}",
                std::str::from_utf8(input)
            );
        }
    }

    #[test]
    fn test_invalid_utf8_content_type() {
        let invalid_data = b"\xFF\xFE\xFD";
        assert!(matches!(
            ContentType::from_header_value(invalid_data),
            ContentType::Unknown
        ));
    }
}
