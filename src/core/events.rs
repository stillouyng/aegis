use super::structs::RequestMeta;
use crate::core::response::Response;
use std::net::SocketAddr;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Event {
    RequestStart {
        method: String,
        path: String,
        version: u8,
        rest: Vec<u8>,
        meta: RequestMeta,
        resp_tx: tokio::sync::oneshot::Sender<Response>,
    },
    RequestBody {
        body: Vec<u8>,
        more_body: bool,
    },
    Disconnect {
        client_addr: SocketAddr,
    },
}
