use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

use super::structs::RequestMeta;
use super::enums::ContentType;
use super::events::Event;

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
        } else if name.eq_ignore_ascii_case(b"transfer-encoding") {
            if value.windows(7).any(|w| w.eq_ignore_ascii_case(b"chunked")) {
                meta.is_chunked = true;
            }
        }
    }
    meta
}

pub async fn run_listener(addr: &str, tx: mpsc::Sender<Event>) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    loop {
        let (mut socket, client_addr) = listener.accept().await?;
        println!("Accepted connection from {}", client_addr);

        let tx = tx.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let mut headers_done = false;
            let mut body_bytes_received = 0usize;
            let mut expected_length: Option<usize> = None;
            let mut header_buffer = Vec::new();

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Client {} disconnected", client_addr);
                        let _ = tx.send(Event::Disconnect).await;
                        break;
                    }
                    Ok(n) => {
                        if !headers_done {
                            header_buffer.extend_from_slice(&buf[..n]);
                            if let Some(pos) = header_buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                                let raw_headers = header_buffer[..pos].to_vec();
                                let rest = header_buffer[pos + 4..].to_vec();
                                header_buffer.clear();
                                let headers = parse_raw_headers(&raw_headers);
                                let meta = headers_to_meta(&headers);
                                headers_done = true;
                                body_bytes_received = rest.len();
                                expected_length = meta.content_length;
                                let event = Event::RequestStart { raw_headers, rest, meta };
                                if tx.send(event).await.is_err() {
                                    println!("Receiver dropped, stopping listener for {}", client_addr);
                                    break;
                                }
                            }
                        } else {
                            body_bytes_received += n;
                            let more_body = expected_length
                                .map(|len| body_bytes_received < len)
                                .unwrap_or(true);
                            let body = buf[..n].to_vec();
                            let event = Event::RequestBody { body, more_body };
                            if tx.send(event).await.is_err() {
                                println!("Receiver dropped, stopping listener for {}", client_addr);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading from client {}: {}", client_addr, e);
                        break;
                    }
                }
            }
        });
    }
}