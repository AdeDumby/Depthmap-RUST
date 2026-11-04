use std::net::{TcpListener, TcpStream};
use rustweb::request::{parse_request, ParseError};
use rustweb::response::{Response, StatusCode};
use rustweb::static_files;
use std::path::Path;
use std::io::{Read, Write};

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).unwrap();
    let text = String::from_utf8_lossy(&buffer[..n]);
    println!("{}", text);
    let response = match parse_request(&text) {
        Ok(req) => static_files::serve(Path::new("public"), &req.target),
        Err(ParseError::Malformed) => Response::new(StatusCode::BadRequest),
        Err(ParseError::UnsupportedMethod) => Response::new(StatusCode::NotImplemented),
        Err(ParseError::UnsupportedVersion) => Response::new(StatusCode::VersionNotSupported),
    };
    stream.write_all(&response.to_bytes()).unwrap();

    let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello";
    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening on 127.0.0.1:8080");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection from {}", stream.peer_addr().unwrap());
        handle_connection(stream);
    }
}
