//! Serving static files from a folder.

use crate::response::{Response, StatusCode};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// Returns the `Content-Type` for a path, based on its extension.
fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("txt") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}

/// Builds the response for a request target, serving files from `root`.
pub fn serve(root: &Path, target: &str) -> Response {
    let relative = target.split('?').next().unwrap_or("").trim_start_matches('/');

    let root = match root.canonicalize() {
        Ok(r) => r,
        Err(_) => return Response::new(StatusCode::InternalServerError),
    };
    let mut path = match root.join(relative).canonicalize() {
        Ok(p) => p,
        Err(e) if e.kind() == ErrorKind::NotFound => return Response::new(StatusCode::NotFound),
        Err(_) => return Response::new(StatusCode::Forbidden),
    };

    // Security: the final path must still be inside the root folder.
    if !path.starts_with(&root) {
        return Response::new(StatusCode::Forbidden);
    }
    if path.is_dir() {
        path = path.join("index.html");
    }

    match fs::read(&path) {
        Ok(bytes) => Response::new(StatusCode::Ok).with_body(content_type(&path), bytes),
        Err(e) if e.kind() == ErrorKind::NotFound => Response::new(StatusCode::NotFound),
        Err(_) => Response::new(StatusCode::Forbidden),
    }
}