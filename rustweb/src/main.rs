use rustweb::request::{parse_request, read_head, Method, ParseError};
use rustweb::response::{Response, StatusCode};
use rustweb::static_files;
use std::io::{BufReader, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

fn handle_connection(stream: TcpStream) {
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    loop {
        let head = match read_head(&mut reader) {
            Ok(Some(head)) => head,
            Ok(None) => return, // the client closed the connection
            Err(e) if e.kind() == ErrorKind::InvalidData => {
                let bad = Response::new(StatusCode::BadRequest).with_header("Connection", "close");
                let _ = writer.write_all(&bad.to_bytes());
                return;
            }
            Err(_) => return, // timeout or network error
        };

        let (response, keep_alive) = match parse_request(&head) {
            Ok(req) => {
                let resp = static_files::serve(Path::new("public"), &req.target);
                let resp = if req.method == Method::Head { resp.for_head() } else { resp };
                (resp, req.keep_alive())
            }
            Err(ParseError::Malformed) => (Response::new(StatusCode::BadRequest), false),
            Err(ParseError::UnsupportedMethod) => (Response::new(StatusCode::NotImplemented), false),
            Err(ParseError::UnsupportedVersion) => {
                (Response::new(StatusCode::VersionNotSupported), false)
            }
        };

        let value = if keep_alive { "keep-alive" } else { "close" };
        let response = response.with_header("Connection", value);
        if writer.write_all(&response.to_bytes()).is_err() || !keep_alive {
            return;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening on 127.0.0.1:8080");

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        println!("Connection from {}", stream.peer_addr().unwrap());
        thread::spawn(move || handle_connection(stream));
    }
}