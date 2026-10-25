use super::{REQUEST_BUFFER_BYTES, request_path};
use std::io::{Cursor, Read};

#[test]
fn fragmented_request_line_keeps_complete_destination() {
    let reader = Cursor::new(b"GET /fi").chain(Cursor::new(b"nal.pdf HTTP/1.1\r\n"));
    assert_eq!(request_path(reader).unwrap(), "/final.pdf");
}

#[test]
fn truncated_request_line_is_rejected() {
    let error = request_path(Cursor::new(b"GET /final.pdf")).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn oversized_request_line_is_rejected() {
    let request = format!("GET /{} HTTP/1.1\r\n", "a".repeat(REQUEST_BUFFER_BYTES));
    let error = request_path(Cursor::new(request)).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}
