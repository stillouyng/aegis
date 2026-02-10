use httparse::{Header, Request, Status};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::core::events::Event;
use crate::core::structs::{RequestMeta, ServerConfig};

use super::listener::Listener;

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
            let mut header_buffer = Vec::new();
            let mut headers_done = false;
            let mut body_bytes_received = 0usize;
            let mut expected_length: Option<usize> = None;
            let mut temp_buf = vec![0u8; config.read_buffer_size];

            loop {
                // 1. Reading the tcp-socket.
                let n = match stream.read(&mut temp_buf).await {
                    Ok(0) => {
                        // Client disconnected
                        debug!("Client Disconnected");
                        let _ = tx.send(Event::Disconnect { client_addr }).await;
                        break;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        // Error while reading.
                        error!("Error while listening: {:?}", e);
                        let _ = tx.send(Event::Disconnect { client_addr }).await;
                        break;
                    }
                };

                if !headers_done {
                    // 2. Reading headers.
                    header_buffer.extend_from_slice(&temp_buf[..n]);

                    // CONDITION
                    // If header_buffer is bigger than MAX_PAYLOAD_SIZE.
                    if header_buffer.len() > config.max_payload_size {
                        warn!(
                            received = header_buffer.len(),
                            limit = config.max_payload_size,
                            "Header is too big."
                        );
                        break;
                    }

                    let mut headers = [Header {
                        name: "",
                        value: &[],
                    }; 64];
                    let mut req = Request::new(&mut headers);

                    match req.parse(&header_buffer) {
                        Ok(Status::Complete(amt)) => {
                            headers_done = true;

                            let method = match req.method {
                                Some(m) => m.to_string(),
                                None => {
                                    // Raise a 400.
                                    warn!("Empty method");
                                    break;
                                }
                            };
                            let path = match req.path {
                                Some(p) => p.to_string(),
                                None => {
                                    // Raise a 400.
                                    warn!("Empty path");
                                    break;
                                }
                            };
                            let version = req.version.unwrap_or(1);

                            let meta = RequestMeta::from_headers(req.headers);
                            expected_length = meta.content_length;

                            // CONDITION
                            // IF Content-Length is more than MAX_PAYLOAD_SIZE.
                            if let Some(len) = expected_length {
                                if len > config.max_payload_size {
                                    warn!(len, limit = config.max_payload_size);
                                    break;
                                }
                            }

                            let rest = header_buffer[amt..].to_vec();
                            body_bytes_received = rest.len();

                            // CONDITION
                            // If rest bytes len is bigger than MAX_PAYLOAD_SIZE.
                            if body_bytes_received > config.max_payload_size {
                                warn!(recieved = body_bytes_received);
                                break;
                            }

                            let _ = tx
                                .send(Event::RequestStart {
                                    method,
                                    path,
                                    version,
                                    rest,
                                    meta,
                                })
                                .await;

                            // Request could be stopped
                        }
                        Ok(Status::Partial) => continue,
                        Err(_) => break,
                    }
                } else {
                    // 3. Reading the body.
                    body_bytes_received += n;

                    // CONDITION
                    // If received bytes len is more than MAX_PAYLOAD_SIZE.
                    if body_bytes_received > config.max_payload_size {
                        warn!(recieved = body_bytes_received);
                        break;
                    }

                    let more_body = expected_length
                        .map(|len| body_bytes_received < len)
                        .unwrap_or(false);

                    let event = Event::RequestBody {
                        body: temp_buf[..n].to_vec(),
                        more_body,
                    };
                    let _ = tx.send(event).await;

                    if !more_body {
                        let _ = tx.send(Event::Disconnect { client_addr }).await;
                        // Request could be stopped
                    }
                }
            }
        });
    }
}
