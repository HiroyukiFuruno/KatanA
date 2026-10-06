use super::DocumentSurfaceSource;
use std::sync::Arc;

fn local_source() -> (tempfile::TempDir, DocumentSurfaceSource) {
    let directory = tempfile::tempdir().expect("source directory");
    let path = directory.path().join("owned.pdf");
    std::fs::write(&path, b"%PDF-1.7\nowned source bytes").expect("source file");
    let source = DocumentSurfaceSource::local(&path).expect("local source");
    (directory, source)
}

#[test]
fn local_result_and_its_clone_retain_memory_until_payloads_drop() {
    let (_directory, source) = local_source();
    let memory = Arc::downgrade(source._intake_memory.as_ref().expect("intake owns memory"));
    let cloned = source.clone();
    let descriptor = source.descriptor();
    assert!(descriptor._intake_memory.is_none());
    assert_eq!(descriptor.byte_len(), 0);
    assert!(cloned.byte_len() > 0);
    drop(source);
    assert!(
        memory.upgrade().is_some(),
        "the cloned bytes are still owned"
    );
    drop(cloned);
    assert!(memory.upgrade().is_none(), "the descriptor owns no bytes");
    drop(descriptor);
}

#[test]
fn a_queued_result_retains_memory_until_receiver_discards_it() {
    let (_directory, source) = local_source();
    let memory = Arc::downgrade(source._intake_memory.as_ref().expect("intake owns memory"));
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send(source).expect("queued source");
    drop(sender);
    assert!(memory.upgrade().is_some(), "the queue owns source bytes");
    drop(receiver);
    assert!(memory.upgrade().is_none());
}
