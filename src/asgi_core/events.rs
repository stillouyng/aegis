use super::structs::RequestMeta;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Event {
    RequestStart {
        raw_headers: Vec<u8>,
        rest: Vec<u8>,
        meta: RequestMeta,
    },
    RequestBody {
        body: Vec<u8>,
        more_body: bool,
    },
    Disconnect,
}
