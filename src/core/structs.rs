use dotenvy;
use std::env;
use std::net::SocketAddr;

use super::enums::ContentType;

#[derive(Debug, Default, Clone)]
pub struct RequestMeta {
    pub content_length: Option<usize>,
    pub content_type: Option<ContentType>,
    pub is_chunked: bool,
}

pub struct ServerConfig {
    pub addr: SocketAddr,
    pub max_payload_size: usize,
    pub read_buffer_size: usize,
}

impl RequestMeta {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_headers(headers: &[httparse::Header<'_>]) -> Self {
        let mut meta = Self::new();

        for header in headers {
            if header.name.eq_ignore_ascii_case("content-length") {
                meta.content_length = std::str::from_utf8(header.value)
                    .ok()
                    .and_then(|s| s.trim().parse().ok());
            } else if header.name.eq_ignore_ascii_case("content-type") {
                meta.content_type = Some(ContentType::from_header_value(header.value));
            } else if header.name.eq_ignore_ascii_case("transfer-encoding")
                && header
                    .value
                    .windows(7)
                    .any(|w| w.eq_ignore_ascii_case(b"chunked"))
            {
                meta.is_chunked = true;
            }
        }
        meta
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        if cfg!(test) {
            dotenvy::from_filename("tests/.env.test").ok();
        } else {
            dotenvy::dotenv().ok();
        }

        let addr = env::var("SERVER_ADDR")
            .map(|s| s.parse().expect("Invalid SERVER_ADDR format"))
            .unwrap_or_else(|_| "127.0.0.1:8080".parse().unwrap());

        let max_payload_size = env::var("MAX_PAYLOAD_SIZE")
            .map(|s| {
                let val = s
                    .parse::<usize>()
                    .expect("MAX_PAYLOAD_SIZE must be a valid number (bytes)");
                if val == 0 {
                    panic!("MAX_PAYLOAD_SIZE cannot be 0");
                }
                val
            })
            .unwrap_or(1024 * 1024);

        let read_buffer_size = env::var("READ_BUFFER_SIZE")
            .map(|s| {
                let val = s
                    .parse()
                    .expect("READ_BUFFER_SIZE must be a valid number (bytes)");
                if val == 0 {
                    panic!("READ_BUFFER_SIZE cannot be 0");
                }
                val
            })
            .unwrap_or(8192);

        Self {
            addr,
            max_payload_size,
            read_buffer_size,
        }
    }
}

#[cfg(test)]
mod tests_requestmeta {
    use super::*;
    use httparse::Header;

    #[test]
    fn test_from_headers_full() {
        let headers = [
            Header {
                name: "Content-Length",
                value: b"1024",
            },
            Header {
                name: "Content-Type",
                value: b"application/json",
            },
            Header {
                name: "Transfer-Encoding",
                value: b"chunked",
            },
        ];

        let meta = RequestMeta::from_headers(&headers);

        assert_eq!(meta.content_length, Some(1024));
        assert!(matches!(meta.content_type, Some(ContentType::Json)));
        assert!(meta.is_chunked);
    }

    #[test]
    fn test_from_headers_case_insensitivity() {
        let headers = [
            Header {
                name: "coNTent-leNgTh",
                value: b"512",
            },
            Header {
                name: "CONTENT-TYPE",
                value: b"text/plain",
            },
        ];

        let meta = RequestMeta::from_headers(&headers);

        assert_eq!(meta.content_length, Some(512));
        assert!(matches!(meta.content_type, Some(ContentType::Text)));
    }

    #[test]
    fn test_from_headers_empty_or_garbage() {
        let headers = [
            Header {
                name: "content-length",
                value: b"not-a-number",
            },
            Header {
                name: "x-custom-header",
                value: b"hello",
            },
        ];

        let meta = RequestMeta::from_headers(&headers);

        // Должно быть None, но не паника
        assert_eq!(meta.content_length, None);
        assert!(meta.content_type.is_none());
        assert!(!meta.is_chunked);
    }

    #[test]
    fn test_from_headers_multiple_payloads() {
        let headers = [
            Header {
                name: "content-length",
                value: b"100",
            },
            Header {
                name: "content-length",
                value: b"200",
            },
        ];

        let meta = RequestMeta::from_headers(&headers);
        assert_eq!(meta.content_length, Some(200));
    }
}

#[cfg(test)]
mod tests_serverconfig {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn setup_envs() {
        env::set_var("SERVER_ADDR", "127.0.0.1:8000");
        env::set_var("MAX_PAYLOAD_SIZE", "1048576");
        env::set_var("READ_BUFFER_SIZE", "8192");
    }

    fn remove_env() {
        env::remove_var("SERVER_ADDR");
        env::remove_var("READ_BUFFER_SIZE");
        env::remove_var("MAX_PAYLOAD_SIZE");
    }

    // If env is empty
    #[test]
    #[serial(env)]
    fn test_config_defaults() {
        remove_env();

        let config = ServerConfig::from_env();

        assert_eq!(config.addr, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(config.read_buffer_size, 8192);
        assert_eq!(config.max_payload_size, 1048576);
    }

    // READ_BUFFER_SIZE has incorrect value
    #[test]
    #[serial(env)]
    #[should_panic(expected = "READ_BUFFER_SIZE")]
    fn test_config_invalid_read_buffer() {
        setup_envs();
        env::set_var("READ_BUFFER_SIZE", "1 MB");

        ServerConfig::from_env();
    }

    // MAX_PAYLOAD_SIZE has incorrect value
    #[test]
    #[serial(env)]
    #[should_panic(expected = "MAX_PAYLOAD_SIZE")]
    fn test_config_invalid_payload_panics() {
        setup_envs();
        env::set_var("MAX_PAYLOAD_SIZE", "one_gigabyte");

        ServerConfig::from_env();
    }

    // SERVER_ADDR has incorrect value
    #[test]
    #[serial(env)]
    #[should_panic(expected = "SERVER_ADDR")]
    fn test_config_invalid_addr() {
        setup_envs();
        env::set_var("SERVER_ADDR", "127.0.0.1.8080");

        ServerConfig::from_env();
    }

    // Check custom correct values.
    #[test]
    #[serial(env)]
    fn test_config_custom_values() {
        setup_envs();

        let config = ServerConfig::from_env();

        assert_eq!(config.addr, "127.0.0.1:8000".parse().unwrap());
        assert_eq!(config.read_buffer_size, 8192);
        assert_eq!(config.max_payload_size, 1048576);
    }

    // Test edge case (0) in READ_BUFFER_SIZE
    #[test]
    #[serial(env)]
    #[should_panic(expected = "READ_BUFFER_SIZE")]
    fn test_edge_zero_cases_read_buffer() {
        setup_envs();
        env::set_var("READ_BUFFER_SIZE", "0");

        ServerConfig::from_env();
    }

    // Test edge case (0) in MAX_PAYLOAD_SIZE.
    #[test]
    #[serial(env)]
    #[should_panic(expected = "MAX_PAYLOAD_SIZE")]
    fn test_edge_zero_case_max_payload() {
        setup_envs();
        env::set_var("MAX_PAYLOAD_SIZE", "0");

        ServerConfig::from_env();
    }
}
