use std::net::SocketAddr;

use super::structs::RequestMeta;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Event {
    RequestStart {
        method: String,
        path: String,
        version: u8,
        rest: Vec<u8>,
        meta: RequestMeta,
    },
    RequestBody {
        body: Vec<u8>,
        more_body: bool,
    },
    Disconnect {
        client_addr: SocketAddr,
    },
}
