use katana_core::document::{Document, DocumentError};

#[test]
fn new_document_is_clean() {
    let doc = Document::new("/tmp/test.md", "# Hello");
    assert!(!doc.is_dirty);
    assert_eq!(doc.buffer, "# Hello");
}

#[test]
fn update_buffer_marks_dirty() {
    let mut doc = Document::new("/tmp/test.md", "# Hello");
    doc.update_buffer("# Hello World");
    assert!(doc.is_dirty);
    assert_eq!(doc.buffer, "# Hello World");
}

#[test]
fn mark_clean_clears_dirty_flag() {
    let mut doc = Document::new("/tmp/test.md", "# Hello");
    doc.update_buffer("# Changed");
    doc.mark_clean();
    assert!(!doc.is_dirty);
}

#[test]
fn update_with_same_content_does_not_dirty() {
    let mut doc = Document::new("/tmp/test.md", "# Hello");
    doc.update_buffer("# Hello");
    assert!(!doc.is_dirty);
}

#[test]
fn file_name_returns_basename() {
    let doc = Document::new("/workspace/spec.md", "");
    assert_eq!(doc.file_name(), Some("spec.md"));
}

#[test]
fn file_name_returns_none_for_root_path() {
    let doc = Document::new("/", "");
    assert_eq!(doc.file_name(), None);
}

#[test]
fn document_error_save_failed() {
    let source = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let err = DocumentError::save_failed("/readonly/file.md", source);
    let msg = err.to_string();
    assert!(msg.contains("readonly") || msg.contains("save") || msg.contains("Save"));
}

#[test]
fn document_error_read_failed() {
    let source = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = DocumentError::read_failed("/nonexistent/file.md", source);
    let msg = err.to_string();
    assert!(msg.contains("nonexistent") || msg.contains("read") || msg.contains("Read"));
}
