use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Arc;

use katana_core::canvas::NodeContent;
use katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES;

use super::tests::RequestTest;

const MULTI_CHUNK_SOURCE_BYTES: usize = 128 * 1024;
const CANCEL_ON_FIRST_READ_COMPLETION_CHECK: usize = 9;
use super::{CanvasParseRequest, CanvasRequestError, input::CanvasInput, pool::CanvasPool};

struct PathTest;

impl PathTest {
    fn source(text: &str) -> String {
        serde_json::json!({"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":40,"text":text}]}).to_string()
    }

    fn request(path: PathBuf, generation: u64) -> CanvasParseRequest {
        CanvasParseRequest::start_path(generation, path, None).expect("path request admitted")
    }
}

#[test]
fn actual_canvas_file_is_loaded_on_worker_and_changed_file_is_reread() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    std::fs::write(&path, PathTest::source("first")).expect("initial file");
    let first = PathTest::request(path.clone(), 1);
    let first = RequestTest::receive(&first);
    assert_eq!(first.generation, 1);
    assert_eq!(
        first.result.expect("first model").nodes[0].content,
        NodeContent::Text {
            text: "first".into()
        }
    );
    std::fs::write(&path, PathTest::source("changed")).expect("changed file");
    let second = PathTest::request(path, 2);
    let second = RequestTest::receive(&second);
    assert_eq!(second.generation, 2);
    assert_eq!(
        second.result.expect("second model").nodes[0].content,
        NodeContent::Text {
            text: "changed".into()
        }
    );
}

#[test]
fn nonexistent_path_is_admitted_then_reports_worker_read_error() {
    let directory = tempfile::tempdir().expect("source directory");
    let request = PathTest::request(directory.path().join("missing.canvas"), 3);
    assert!(matches!(
        RequestTest::receive(&request).result,
        Err(CanvasRequestError::FileRead(std::io::ErrorKind::NotFound))
    ));
}

#[test]
fn invalid_utf8_is_reported_before_json_parsing() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    std::fs::write(&path, [u8::MAX]).expect("invalid UTF8 file");
    let request = PathTest::request(path, 4);
    assert!(matches!(
        RequestTest::receive(&request).result,
        Err(CanvasRequestError::InvalidUtf8)
    ));
}

#[test]
fn directory_path_is_rejected_as_non_regular_file() {
    let directory = tempfile::tempdir().expect("source directory");
    let request = PathTest::request(directory.path().to_owned(), 5);
    assert!(matches!(
        RequestTest::receive(&request).result,
        Err(CanvasRequestError::NotRegularFile)
    ));
}

#[cfg(unix)]
#[test]
fn real_socket_path_is_rejected_before_open_read() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    let _listener = std::os::unix::net::UnixListener::bind(&path).expect("socket path");
    let request = PathTest::request(path, 6);
    assert!(matches!(
        RequestTest::receive(&request).result,
        Err(CanvasRequestError::NotRegularFile)
    ));
}

#[test]
fn sparse_oversize_file_is_rejected_without_allocating_file_content() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    std::fs::File::create(&path)
        .expect("sparse file")
        .set_len(MAX_BINARY_DOCUMENT_BYTES as u64 + 1)
        .expect("oversize metadata");
    let request = PathTest::request(path, 7);
    assert!(matches!(
        RequestTest::receive(&request).result,
        Err(CanvasRequestError::SizeLimit)
    ));
}

#[test]
fn real_file_read_cooperatively_cancels_after_first_bounded_chunk() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    std::fs::write(
        &path,
        PathTest::source(&"x".repeat(MULTI_CHUNK_SOURCE_BYTES)),
    )
    .expect("multi-chunk file");
    let checks = Cell::new(0);
    let result = CanvasInput::File(path).load(|| {
        let count = checks.get() + 1;
        checks.set(count);
        count >= CANCEL_ON_FIRST_READ_COMPLETION_CHECK
    });
    assert_eq!(result, Err(CanvasRequestError::Cancelled));
    assert!(checks.get() >= CANCEL_ON_FIRST_READ_COMPLETION_CHECK);
}

#[test]
fn oversize_memory_source_uses_the_same_limit_as_file_input() {
    let source = " ".repeat(MAX_BINARY_DOCUMENT_BYTES + 1);
    assert!(matches!(
        CanvasInput::Text(source).load(|| false),
        Err(CanvasRequestError::SizeLimit)
    ));
}

#[test]
fn dropped_queued_file_request_releases_path_owner_and_is_not_opened() {
    let pool = CanvasPool::start().expect("worker start");
    let directory = tempfile::tempdir().expect("source directory");
    let barrier = RequestTest::blocked(&pool, || {
        let request = CanvasParseRequest::submit_input(
            &pool,
            8,
            CanvasInput::File(directory.path().join("missing.canvas")),
            None,
        )
        .expect("path queue");
        let weak = Arc::downgrade(&request.request);
        drop(request);
        assert!(weak.upgrade().is_none());
        RequestTest::submit(&pool, 9, "{}")
    });
    assert!(RequestTest::receive(&barrier).result.is_ok());
}
