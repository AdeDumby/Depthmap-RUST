//! HTTP request parsing.
use std::io::{self, BufRead};
use std::collections::HashMap;

/// An HTTP request method.
#[derive(Debug, PartialEq)]
pub enum Method {
    /// The `GET` method.
    Get,
    /// The `HEAD` method.
    Head,
}

/// An HTTP protocol version.
#[derive(Debug, PartialEq)]
pub enum Version {
    /// HTTP/1.0
    Http10,
    /// HTTP/1.1
    Http11,
}

/// Reasons a request can be rejected while parsing.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The request is not well formed (maps to 400).
    Malformed,
    /// The method is not supported (maps to 501).
    UnsupportedMethod,
    /// The HTTP version is not supported (maps to 505).
    UnsupportedVersion,
}

/// The first line of a request, e.g. `GET /index.html HTTP/1.1`.
#[derive(Debug)]
pub struct RequestLine {
    /// The request method.
    pub method: Method,
    /// The requested resource, e.g. `/index.html`.
    pub target: String,
    /// The protocol version.
    pub version: Version,
}

/// Parses a request line such as `GET /index.html HTTP/1.1`.
pub fn parse_request_line(line: &str) -> Result<RequestLine, ParseError> {
    let mut parts = line.split(' ');
    let method = parts.next().ok_or(ParseError::Malformed)?;
    let target = parts.next().ok_or(ParseError::Malformed)?;
    let version = parts.next().ok_or(ParseError::Malformed)?;
    if parts.next().is_some() {
        return Err(ParseError::Malformed);
    }

    let method = match method {
        "GET" => Method::Get,
        "HEAD" => Method::Head,
        _ => return Err(ParseError::UnsupportedMethod),
    };
    let version = match version {
        "HTTP/1.0" => Version::Http10,
        "HTTP/1.1" => Version::Http11,
        _ => return Err(ParseError::UnsupportedVersion),
    };

    Ok(RequestLine { method, target: target.to_string(), version })
}

/// A parsed HTTP request.
#[derive(Debug)]
pub struct Request {
    /// The request method.
    pub method: Method,
    /// The requested resource, e.g. `/index.html`.
    pub target: String,
    /// The protocol version.
    pub version: Version,
    /// The request headers. Names are stored in lowercase.
    pub headers: HashMap<String, String>,
}

impl Request {
    /// Returns the value of a header, looking it up case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|v| v.as_str())
    }
}

/// Parses a complete request head (request line plus headers) from text.
pub fn parse_request(text: &str) -> Result<Request, ParseError> {
    let mut lines = text.split("\r\n");
    let first = lines.next().ok_or(ParseError::Malformed)?;
    let RequestLine { method, target, version } = parse_request_line(first)?;

    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':').ok_or(ParseError::Malformed)?;
        headers.insert(name.trim().to_lowercase(), value.trim().to_string());
    }

    Ok(Request { method, target, version, headers })

}

/// Reads one request head (request line plus headers) from a reader.
///
/// Returns `Ok(None)` if the client closed the connection before
/// sending a complete head.
pub fn read_head<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let end_of_head = line == "\r\n" || line == "\n";
        head.push_str(&line);
        if end_of_head {
            return Ok(Some(head));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_get() {
        let rl = parse_request_line("GET /index.html HTTP/1.1").unwrap();
        assert_eq!(rl.method, Method::Get);
        assert_eq!(rl.target, "/index.html");
        assert_eq!(rl.version, Version::Http11);
    }

        #[test]
    fn reads_until_blank_line() {
        let mut input = &b"GET / HTTP/1.1\r\nHost: x\r\n\r\nBODY"[..];
        let head = read_head(&mut input).unwrap().unwrap();
        assert_eq!(head, "GET / HTTP/1.1\r\nHost: x\r\n\r\n");
    }

    #[test]
    fn empty_input_means_closed() {
        let mut input = &b""[..];
        assert!(read_head(&mut input).unwrap().is_none());
    }
}