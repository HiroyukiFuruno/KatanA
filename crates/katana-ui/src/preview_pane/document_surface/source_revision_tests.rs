use super::super::source::DocumentSurfaceSource;
use super::{IntakeOutcome, SourceRevision, SourceRevisionOps};

const MUTATION_FIXTURE_BYTES: usize = 128 * 1024;

#[test]
fn unchanged_revision_returns_no_owned_document_payload() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("revision.pdf");
    std::fs::write(&path, b"%PDF-1.7\nunchanged").unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    let result = SourceRevisionOps::load(&path, Some(&known), || false).unwrap();
    assert!(matches!(result, IntakeOutcome::Unchanged(actual) if actual == known));
}

#[test]
fn same_size_same_mtime_change_loads_actual_new_payload() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("revision.pdf");
    let first = b"%PDF-1.7\nversion-one";
    let second = b"%PDF-1.7\nversion-two";
    std::fs::write(&path, first).unwrap();
    let modified = path.metadata().unwrap().modified().unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    std::fs::write(&path, second).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert_eq!(path.metadata().unwrap().modified().unwrap(), modified);
    let IntakeOutcome::Loaded(mut updated) =
        SourceRevisionOps::load(&path, Some(&known), || false).unwrap()
    else {
        panic!("changed source must load")
    };
    assert_eq!(updated.revision, super::super::source_io::revision(second));
    assert_eq!(updated.take_bytes().unwrap(), second);
}

#[test]
fn initial_and_manual_requests_without_known_revision_keep_full_payload() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("initial.pdf");
    let bytes = b"%PDF-1.7\ninitial";
    std::fs::write(&path, bytes).unwrap();
    let IntakeOutcome::Loaded(mut source) = SourceRevisionOps::load(&path, None, || false).unwrap()
    else {
        panic!("initial request must load")
    };
    assert_eq!(source.take_bytes().unwrap(), bytes);
}

#[test]
fn invalid_changed_source_and_cancelled_checks_remain_failures() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("invalid.pdf");
    std::fs::write(&path, b"%PDF-1.7\noriginal").unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    assert!(SourceRevisionOps::load(&path, Some(&known), || true).is_err());
    std::fs::write(&path, b"not a PDF document").unwrap();
    let error = SourceRevisionOps::load(&path, Some(&known), || false).unwrap_err();
    assert_eq!(error.operation, "validate");
}

#[test]
fn different_canonical_uri_cannot_use_another_sources_revision() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.pdf");
    let second = directory.path().join("second.pdf");
    std::fs::write(&first, b"%PDF-1.7\nsame").unwrap();
    std::fs::copy(&first, &second).unwrap();
    let source = DocumentSurfaceSource::local(&first).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    let IntakeOutcome::Loaded(updated) =
        SourceRevisionOps::load(&second, Some(&known), || false).unwrap()
    else {
        panic!("different URI must load")
    };
    assert_ne!(updated.uri, known.uri);
}

#[test]
fn mutation_during_stream_uses_the_actual_reloaded_payload() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("changing.pdf");
    let mut first = vec![b'a'; MUTATION_FIXTURE_BYTES];
    first[..b"%PDF-1.7".len()].copy_from_slice(b"%PDF-1.7");
    let mut second = first.clone();
    second.fill(b'b');
    second[..b"%PDF-1.7".len()].copy_from_slice(b"%PDF-1.7");
    std::fs::write(&path, &first).unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    let checks = std::cell::Cell::new(0);
    let result = SourceRevisionOps::load(&path, Some(&known), || {
        let count = checks.get();
        checks.set(count + 1);
        if count == 2 {
            std::fs::write(&path, &second).unwrap();
        }
        false
    })
    .unwrap();
    let IntakeOutcome::Loaded(mut updated) = result else {
        panic!("changed stream must reload")
    };
    assert_eq!(updated.revision, super::super::source_io::revision(&second));
    assert_eq!(updated.take_bytes().unwrap(), second);
}

#[cfg(unix)]
#[test]
fn known_revision_fifo_refresh_consumes_only_one_writer() {
    use std::io::Write;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("single-writer.pdf");
    let bytes = b"%PDF-1.7\nsingle writer";
    std::fs::write(&path, bytes).unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    std::fs::rename(&path, directory.path().join("original.pdf")).unwrap();
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader_path = path.clone();
    let reader = std::thread::spawn(move || {
        let result = SourceRevisionOps::load(&reader_path, Some(&known), || false);
        sender.send(result).unwrap();
    });
    let writer = std::thread::spawn(move || {
        std::fs::File::create(path)
            .unwrap()
            .write_all(bytes)
            .unwrap();
    });
    let IntakeOutcome::Loaded(mut loaded) = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap()
        .unwrap()
    else {
        panic!("special source must use original full reader")
    };
    assert_eq!(loaded.take_bytes().unwrap(), bytes);
    reader.join().unwrap();
    writer.join().unwrap();
}
