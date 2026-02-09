use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::core::enums::ContentType;
use crate::core::events::Event;
use crate::core::structs::{RequestMeta, ServerConfig};

use super::listener::Listener;

type Headers = Vec<(Vec<u8>, Vec<u8>)>;

fn parse_raw_headers(raw: &[u8]) -> Headers {
    let mut headers = Vec::new();
    for line in raw.split(|&b| b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if let Some(colon) = line.iter().position(|&b| b == b':') {
            let name = line[..colon].to_vec();
            let value = line[colon + 1..].to_vec();
            headers.push((name, value));
        }
    }
    headers
}

fn parse_content_length(value: &[u8]) -> Option<usize> {
    value
        .iter()
        .filter(|&&b| b != b' ')
        .try_fold(0usize, |acc, &b| {
            if b.is_ascii_digit() {
                Some(acc * 10 + (b - b'0') as usize)
            } else {
                None
            }
        })
}

fn headers_to_meta(headers: &Headers) -> RequestMeta {
    let mut meta = RequestMeta::new();
    for (name, value) in headers {
        if name.eq_ignore_ascii_case(b"content-length") {
            meta.content_length = parse_content_length(value);
        } else if name.eq_ignore_ascii_case(b"content-type") {
            meta.content_type = Some(ContentType::from_header_value(value));
        } else if name.eq_ignore_ascii_case(b"transfer-encoding")
            && value.windows(7).any(|w| w.eq_ignore_ascii_case(b"chunked"))
        {
            meta.is_chunked = true;
        }
    }
    meta
}

pub async fn run_server(
    listener: Box<dyn Listener>,
    tx: mpsc::Sender<Event>,
    config: Arc<ServerConfig>,
) -> tokio::io::Result<()> {
    loop {
        let (mut stream, client_addr) = listener.accept().await?;
        let tx = tx.clone();
        let config = Arc::clone(&config);
        let connection_span = tracing::info_span!("http_conn", client = %client_addr);
        let _enter = connection_span.enter();

        debug!("Accepted connection");

        tokio::spawn(async move {
            let mut buf = vec![0u8; config.read_buffer_size];
            let mut headers_done = false;
            let mut body_bytes_received = 0usize;
            let mut expected_length: Option<usize> = None;
            let mut header_buffer = Vec::new();

            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        debug!("Client disconnected");
                        let _ = tx.send(Event::Disconnect { client_addr }).await;
                        break;
                    }
                    Ok(n) => {
                        if !headers_done {
                            header_buffer.extend_from_slice(&buf[..n]);

                            // If only header is bigger than MAX_PAYLOAD_SIZE.
                            if header_buffer.len() > config.max_payload_size {
                                warn!(
                                    received = header_buffer.len(),
                                    limit = config.max_payload_size,
                                    "Header is too big."
                                );
                                break;
                            }

                            if let Some(pos) =
                                header_buffer.windows(4).position(|w| w == b"\r\n\r\n")
                            {
                                let raw_headers = header_buffer[..pos].to_vec();
                                let rest = header_buffer[pos + 4..].to_vec();
                                header_buffer.clear();

                                let headers = parse_raw_headers(&raw_headers);
                                let meta = headers_to_meta(&headers);

                                headers_done = true;
                                body_bytes_received = rest.len();
                                expected_length = meta.content_length;

                                // If Content-Length more than MAX_PAYLOAD_SIZE.
                                if let Some(len) = expected_length {
                                    if len > config.max_payload_size {
                                        warn!(len, limit = config.max_payload_size);
                                        break;
                                    }
                                }

                                // If rest bytes more than MAX_PAYLOAD_SIZE.
                                if body_bytes_received > config.max_payload_size {
                                    warn!(recieved = body_bytes_received);
                                    break;
                                }
                                let event = Event::RequestStart {
                                    raw_headers,
                                    rest,
                                    meta,
                                };
                                if tx.send(event).await.is_err() {
                                    debug!("Receiver dropped, stopping listener");
                                    break;
                                }
                            }
                        } else {
                            body_bytes_received += n;

                            // If body_bytes_received more than MAX_PAYLOAD_SIZE.
                            if body_bytes_received > config.max_payload_size {
                                warn!(
                                    received = body_bytes_received,
                                    limit = config.max_payload_size,
                                    "Payload limit exceeded during streaming!"
                                );
                                break;
                            }

                            let more_body = expected_length
                                .map(|len| body_bytes_received < len)
                                .unwrap_or(false);
                            let body = buf[..n].to_vec();
                            let event = Event::RequestBody { body, more_body };
                            if tx.send(event).await.is_err() {
                                debug!("Receiver dropped, stopping listener");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
