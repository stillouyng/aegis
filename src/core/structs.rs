use dotenvy::dotenv;
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
}

impl ServerConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        let addr = env::var("SERVER_ADDR")
            .ok()
            .and_then(|s| s.parse::<SocketAddr>().ok())
            .unwrap_or_else(|| "127.0.0.1:8080".parse().unwrap());
        let max_payload_size = env::var("MAX_PAYLOAD_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1024 * 1024);

        let read_buffer_size = env::var("READ_BUFFER_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8192);

        Self {
            addr,
            max_payload_size,
            read_buffer_size,
        }
    }
}
