use super::tests::RequestTest;
use super::{CanvasParseRequest, CanvasRequestError};

#[test]
fn repeated_file_reads_share_revision_and_equal_size_changed_bytes_do_not() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    let original = r#"{"nodes":[]}"#;
    let changed = r#"{"edges":[]}"#;
    assert_eq!(original.len(), changed.len());
    std::fs::write(&path, original).expect("original file");
    let first = CanvasParseRequest::start_path(1, path.clone(), None).expect("first request");
    let first = RequestTest::receive(&first);
    let second = CanvasParseRequest::start_path(2, path.clone(), None).expect("second request");
    let second = RequestTest::receive(&second);
    assert!(first.revision.is_some());
    assert_eq!(first.revision, second.revision);
    std::fs::write(&path, changed).expect("equal-size changed file");
    let third = CanvasParseRequest::start_path(3, path, None).expect("changed request");
    let third = RequestTest::receive(&third);
    assert_ne!(first.revision, third.revision);
    assert_eq!(
        first.result.expect("first model"),
        third.result.expect("changed model")
    );
}

#[test]
fn identical_text_and_file_bytes_have_the_same_source_revision() {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("source.canvas");
    let source = " {\"nodes\":[]}\n";
    std::fs::write(&path, source).expect("source file");
    let text = CanvasParseRequest::start(4, source.to_owned(), None).expect("text request");
    let file = CanvasParseRequest::start_path(5, path, None).expect("file request");
    assert_eq!(
        RequestTest::receive(&text).revision,
        RequestTest::receive(&file).revision
    );
}

#[test]
fn missing_file_failure_has_no_source_revision() {
    let directory = tempfile::tempdir().expect("source directory");
    let request = CanvasParseRequest::start_path(6, directory.path().join("missing.canvas"), None)
        .expect("request");
    let result = RequestTest::receive(&request);
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::FileRead(_))
    ));
    assert_eq!(result.revision, None);
}

#[test]
fn malformed_utf8_json_source_keeps_revision_with_parse_failure() {
    let request = CanvasParseRequest::start(7, "{broken".to_owned(), None).expect("request");
    let result = RequestTest::receive(&request);
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::InvalidCanvas(_))
    ));
    assert!(result.revision.is_some());
}
