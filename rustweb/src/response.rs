//! HTTP responses.

use std::time::SystemTime;

/// HTTP status codes supported by the server.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusCode {
    /// 200 OK
    Ok,
    /// 400 Bad Request
    BadRequest,
    /// 403 Forbidden
    Forbidden,
    /// 404 Not Found
    NotFound,
    /// 405 Method Not Allowed
    MethodNotAllowed,
    /// 500 Internal Server Error
    InternalServerError,
    /// 501 Not Implemented
    NotImplemented,
    /// 505 HTTP Version Not Supported
    VersionNotSupported,
}

impl StatusCode {
    /// The numeric code, e.g. `404`.
    pub fn code(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::BadRequest => 400,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::VersionNotSupported => 505,
        }
    }

    /// The standard reason phrase, e.g. `Not Found`.
    pub fn reason(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::VersionNotSupported => "HTTP Version Not Supported",
        }
    }
}

/// An HTTP response.
#[derive(Debug)]
pub struct Response {
    /// The status code.
    pub status: StatusCode,
    /// Extra headers (`Server`, `Date` and `Content-Length` are added automatically).
    pub headers: Vec<(String, String)>,
    /// The body bytes.
    pub body: Vec<u8>,
}

impl Response {
    /// Creates an empty response with the given status.
    pub fn new(status: StatusCode) -> Self {
        Response { status, headers: Vec::new(), body: Vec::new() }
    }

    /// Sets the body and its `Content-Type`.
    pub fn with_body(mut self, content_type: &str, body: Vec<u8>) -> Self {
        self.headers.push(("Content-Type".to_string(), content_type.to_string()));
        self.body = body;
        self
    }

    /// Serializes the response to the bytes sent on the wire.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status.code(), self.status.reason());
        head.push_str("Server: rustweb/0.1\r\n");
        head.push_str(&format!("Date: {}\r\n", httpdate::fmt_http_date(SystemTime::now())));
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str(&format!("Content-Length: {}\r\n\r\n", self.body.len()));

        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_status_line_and_length() {
        let bytes = Response::new(StatusCode::NotFound)
            .with_body("text/plain", b"nope".to_vec())
            .to_bytes();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(text.contains("Content-Length: 4\r\n"));
        assert!(text.ends_with("\r\n\r\nnope"));
    }
}