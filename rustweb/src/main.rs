use rustweb::request::{parse_request, read_head, Method, ParseError};
use rustweb::response::{Response, StatusCode};
use rustweb::static_files;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn handle_connection(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let head = match read_head(&mut reader) {
        Ok(Some(head)) => head,
        Ok(None) => return,
        Err(_) => {
            let _ = stream.write_all(&Response::new(StatusCode::BadRequest).to_bytes());
            return;
        }
    };

    let response = match parse_request(&head) {
        Ok(req) => {
            let resp = static_files::serve(Path::new("public"), &req.target);
            if req.method == Method::Head { resp.for_head() } else { resp }
        }
        Err(ParseError::Malformed) => Response::new(StatusCode::BadRequest),
        Err(ParseError::UnsupportedMethod) => Response::new(StatusCode::NotImplemented),
        Err(ParseError::UnsupportedVersion) => Response::new(StatusCode::VersionNotSupported),
    };
    stream.write_all(&response.to_bytes()).unwrap();
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