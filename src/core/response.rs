use crate::core::enums::HttpStatus;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub version: String, // HTTP/1.1
}

impl Response {
    pub fn new(status: HttpStatus) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Server".to_string(), "Aegis/0.1".to_string());

        Self {
            status,
            headers,
            body: Vec::new(),
            version: "HTTP/1.1".to_string(),
        }
    }

    // Builder-pattern
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.headers
            .insert("Content-Length".to_string(), body.len().to_string());
        self.body = body;
        self
    }

    pub fn build(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(256 + self.body.len());

        // 1. Status line
        res.extend_from_slice(self.version.as_bytes()); // "HTTP/1.1"
        res.push(b' ');
        res.extend_from_slice(self.status.as_bytes()); // "Status as number"
        res.push(b' ');
        res.extend_from_slice(self.status.text().as_bytes()); // "Status as text"
        res.extend_from_slice(b"\r\n");

        // 2. Headers
        for (name, value) in &self.headers {
            res.extend_from_slice(name.as_bytes());
            res.extend_from_slice(b": ");
            res.extend_from_slice(value.as_bytes());
            res.extend_from_slice(b"\r\n");
        }

        // 3. An empty string before the body
        res.extend_from_slice(b"\r\n");

        // 4. Body
        res.extend_from_slice(&self.body);

        res
    }

    pub fn ok() -> Self {
        Self::new(HttpStatus::Ok)
    }
    pub fn not_found() -> Self {
        Self::new(HttpStatus::NotFound)
    }
    pub fn bad_request() -> Self {
        Self::new(HttpStatus::BadRequest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::enums::HttpStatus;

    #[test]
    fn test_response_build_basic() {
        let response = Response::ok().body(b"Hello Aegis".to_vec());

        let raw = response.build();
        let s = String::from_utf8(raw).unwrap();

        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Server: Aegis/0.1\r\n"));
        assert!(s.contains("Content-Length: 11\r\n"));
        assert!(s.contains("\r\n\r\nHello Aegis"));
    }

    #[test]
    fn test_custom_headers() {
        let response = Response::new(HttpStatus::NotFound)
            .header("X-Custom", "TestValue")
            .body(vec![]);

        let raw = response.build();
        let s = String::from_utf8(raw).unwrap();

        assert!(s.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(s.contains("X-Custom: TestValue\r\n"));
        assert!(s.contains("Content-Length: 0\r\n"));
    }

    #[test]
    fn test_empty_body() {
        let raw = Response::ok().build();
        let s = String::from_utf8(raw).unwrap();

        assert!(s.ends_with("\r\n\r\n"));
    }

    #[test]
    fn test_crlf_structure() {
        let raw = Response::ok().body(b"abc".to_vec()).build();

        let raw_str = String::from_utf8(raw.clone()).unwrap();

        assert!(raw_str.contains("\r\n\r\n"));

        let _parts: Vec<&[u8]> = raw.split(|&b| b == b'\r').collect();
        assert!(raw.ends_with(b"abc"));
    }

    #[test]
    fn test_binary_body() {
        let body = vec![0, 159, 255, 88, 42];

        let raw = Response::ok().body(body.clone()).build();

        assert!(raw.ends_with(&body));
    }

    #[test]
    fn test_content_length_overwrite() {
        let response = Response::ok()
            .header("Content-Length", "999")
            .body(b"abc".to_vec());

        let raw = String::from_utf8(response.build()).unwrap();

        assert!(raw.contains("Content-Length: 3\r\n"));
    }

    #[test]
    fn test_server_header_present() {
        let raw = String::from_utf8(Response::ok().build()).unwrap();
        assert!(raw.contains("Server: Aegis/0.1\r\n"));
    }

    #[test]
    fn test_status_line_exact() {
        let raw = Response::not_found().build();
        let raw_str = String::from_utf8(raw).unwrap();

        let first_line = raw_str.lines().next().unwrap();
        assert_eq!(first_line, "HTTP/1.1 404 Not Found");
    }
}
