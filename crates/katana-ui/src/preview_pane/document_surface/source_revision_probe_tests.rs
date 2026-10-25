use super::SourceRevisionProbe;
use std::fs::File;

const TEST_LIMIT: usize = 16;

#[test]
fn streamed_hash_matches_owned_hash_and_enforces_actual_file_size() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bounded.pdf");
    let bytes = b"%PDF-1.7\nsmall";
    std::fs::write(&path, bytes).unwrap();
    let (revision, count) =
        SourceRevisionProbe::hash(File::open(&path).unwrap(), &path, TEST_LIMIT).unwrap();
    assert_eq!(revision, super::super::super::source_io::revision(bytes));
    assert_eq!(count, bytes.len());
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(TEST_LIMIT as u64 + 1)
        .unwrap();
    let error =
        SourceRevisionProbe::hash(File::open(&path).unwrap(), &path, TEST_LIMIT).unwrap_err();
    assert_eq!(error.operation, "validate size");
    assert!(SourceRevisionProbe::hash(File::open(&path).unwrap(), &path, usize::MAX).is_err());
}

#[test]
fn cancellation_after_real_stream_read_remains_visible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cancel.pdf");
    std::fs::write(&path, b"%PDF-1.7\ncancel").unwrap();
    let source = super::super::super::source::DocumentSurfaceSource::local(&path).unwrap();
    let known = super::super::SourceRevision::from_source(&source, 1);
    let checks = std::cell::Cell::new(0);
    let error = SourceRevisionProbe::check(&path, &known, || {
        let count = checks.get();
        checks.set(count + 1);
        count >= 2
    })
    .err()
    .expect("cancelled stream must fail");
    assert_eq!(error.operation, "read");
}

#[test]
fn interrupted_read_retries_but_cancellation_error_does_not() {
    use std::io::{Error, ErrorKind, Read};

    struct FirstErrorReader {
        error: Option<ErrorKind>,
        payload: std::io::Cursor<&'static [u8]>,
    }
    impl Read for FirstErrorReader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if let Some(error) = self.error.take() {
                return Err(Error::new(error, "first read"));
            }
            self.payload.read(buffer)
        }
    }
    let path = std::path::Path::new("interrupted.pdf");
    let payload = b"%PDF-1.7";
    let reader = FirstErrorReader {
        error: Some(ErrorKind::Interrupted),
        payload: std::io::Cursor::new(payload.as_slice()),
    };
    assert_eq!(
        SourceRevisionProbe::hash(reader, path, TEST_LIMIT)
            .unwrap()
            .0,
        super::super::super::source_io::revision(payload)
    );
    let reader = FirstErrorReader {
        error: Some(ErrorKind::Other),
        payload: std::io::Cursor::new(payload.as_slice()),
    };
    assert!(SourceRevisionProbe::hash(reader, path, TEST_LIMIT).is_err());
}
