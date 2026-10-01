use std::io::Read;
use std::path::Path;

use katana_core::document_source::BinaryDocumentFormat;
use sha2::{Digest, Sha256};

use super::types::DocumentFailure;

const HEX_NIBBLE_BITS: u8 = 4;
const HEX_NIBBLE_MASK: u8 = 0x0f;

pub(super) fn revision(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let digest = Sha256::digest(bytes);
    let mut revision = String::with_capacity("sha256:".len() + digest.len() * 2);
    revision.push_str("sha256:");
    for byte in digest {
        revision.push(char::from(HEX[usize::from(byte >> HEX_NIBBLE_BITS)]));
        revision.push(char::from(HEX[usize::from(byte & HEX_NIBBLE_MASK)]));
    }
    revision
}

pub(super) fn file_url(
    path: &Path,
    format: BinaryDocumentFormat,
) -> Result<String, DocumentFailure> {
    url::Url::from_file_path(path)
        .map(|url| url.to_string())
        .map_err(|()| {
            DocumentFailure::intake(
                "canonicalize",
                path,
                Some(format),
                "file path cannot be represented as a URL",
            )
        })
}

pub(super) fn read_bounded(
    path: &Path,
    format: Option<BinaryDocumentFormat>,
) -> Result<Vec<u8>, DocumentFailure> {
    read_bounded_with_limit(
        path,
        format,
        katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES,
    )
}

pub(super) fn read_bounded_with_limit(
    path: &Path,
    format: Option<BinaryDocumentFormat>,
    limit: usize,
) -> Result<Vec<u8>, DocumentFailure> {
    let started_at = std::time::Instant::now();
    let file = std::fs::File::open(path)
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    let opened_at = std::time::Instant::now();
    let result = opened_file_capacity(file.metadata(), path, format, limit.saturating_add(1))
        .and_then(|capacity| read_limited(file, path, format, limit, capacity));
    super::debug_log::DebugLog::write(
        "document_file_read",
        format_args!(
            "open_ms={} read_ms={} success={} path={}",
            opened_at.duration_since(started_at).as_millis(),
            opened_at.elapsed().as_millis(),
            result.is_ok(),
            path.display()
        ),
    );
    result
}

fn read_limited(
    reader: impl Read,
    path: &Path,
    format: Option<BinaryDocumentFormat>,
    limit: usize,
    capacity: usize,
) -> Result<Vec<u8>, DocumentFailure> {
    let read_limit = limit.checked_add(1).ok_or_else(|| {
        DocumentFailure::intake(
            "validate size",
            path,
            format,
            "document size limit cannot include an overflow sentinel",
        )
    })?;
    let mut bytes = source_buffer(capacity.min(read_limit), path, format)?;
    reader
        .take(read_limit as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    enforce_size_limit(path, format, bytes.len(), limit)?;
    Ok(bytes)
}

fn opened_file_capacity(
    metadata: std::io::Result<std::fs::Metadata>,
    path: &Path,
    format: Option<BinaryDocumentFormat>,
    read_limit: usize,
) -> Result<usize, DocumentFailure> {
    let metadata = metadata
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    Ok(if metadata.is_file() {
        metadata.len().min(read_limit as u64) as usize
    } else {
        0
    })
}

fn source_buffer(
    capacity: usize,
    path: &Path,
    format: Option<BinaryDocumentFormat>,
) -> Result<Vec<u8>, DocumentFailure> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    Ok(bytes)
}

pub(super) fn enforce_remote_size(
    uri: &str,
    format: Option<BinaryDocumentFormat>,
    actual: usize,
) -> Result<(), DocumentFailure> {
    let limit = katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES;
    if actual <= limit {
        return Ok(());
    }
    Err(DocumentFailure::source_intake(
        "validate size",
        uri,
        format,
        format!("document is {actual} bytes; limit is {limit} bytes"),
    ))
}

fn enforce_size_limit(
    path: &Path,
    format: Option<BinaryDocumentFormat>,
    actual: usize,
    limit: usize,
) -> Result<(), DocumentFailure> {
    if actual <= limit {
        return Ok(());
    }
    Err(DocumentFailure::intake(
        "validate size",
        path,
        format,
        format!("document is {actual} bytes; limit is {limit} bytes"),
    ))
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::{
        BinaryDocumentFormat, enforce_remote_size, file_url, opened_file_capacity, read_bounded,
        read_bounded_with_limit, read_limited, revision, source_buffer,
    };

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("deterministic read failure"))
        }
    }

    #[test]
    fn regular_file_capacity_stays_within_the_read_sentinel() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("capacity.pdf");
        let length = 3 * 1024 * 1024 + 1;
        std::fs::write(&path, vec![0x5a; length]).expect("generated file");

        let bytes = read_bounded_with_limit(&path, Some(BinaryDocumentFormat::Pdf), length)
            .expect("bounded read");

        assert_eq!(bytes.len(), length);
        assert!(bytes.capacity() <= length + 1);
    }

    #[test]
    fn regular_capacity_hint_is_bounded_and_non_regular_hint_is_zero() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("hint.pdf");
        std::fs::write(&path, b"12345678").expect("generated file");
        let file = std::fs::File::open(&path).expect("opened file");
        assert_eq!(
            opened_file_capacity(file.metadata(), &path, None, 5).expect("bounded hint"),
            5
        );
        assert_eq!(
            opened_file_capacity(std::fs::metadata(directory.path()), &path, None, 5)
                .expect("non-regular hint"),
            0
        );
        std::fs::write(&path, b"").expect("truncate file");
        assert_eq!(
            opened_file_capacity(file.metadata(), &path, None, 5).expect("empty hint"),
            0
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_regular_opened_file_remains_a_stream() {
        let bytes = read_bounded_with_limit(std::path::Path::new("/dev/null"), None, 4)
            .expect("non-regular stream");
        assert!(bytes.is_empty());
    }

    #[test]
    fn opened_file_metadata_failure_is_not_silently_ignored() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("missing.pdf");
        let failure = opened_file_capacity(std::fs::metadata(&path), &path, None, 9)
            .expect_err("metadata failure");
        assert_eq!(failure.operation, "read");
        assert!(!failure.cause.is_empty());
    }

    #[test]
    fn capacity_hint_does_not_hide_file_growth_past_the_limit() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("growing.pdf");
        std::fs::write(&path, b"123").expect("generated file");
        let file = std::fs::File::open(&path).expect("opened file");
        let capacity = opened_file_capacity(file.metadata(), &path, None, 5).expect("hint");
        std::fs::write(&path, b"123456789").expect("grow file");

        let failure = read_limited(file, &path, None, 4, capacity).expect_err("size limit");
        assert_eq!(failure.operation, "validate size");
        assert!(failure.cause.contains("5 bytes; limit is 4 bytes"));
    }

    #[test]
    fn capacity_hint_does_not_pad_a_file_that_shrinks() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("shrinking.pdf");
        std::fs::write(&path, b"12345678").expect("generated file");
        let file = std::fs::File::open(&path).expect("opened file");
        let capacity = opened_file_capacity(file.metadata(), &path, None, 9).expect("hint");
        std::fs::write(&path, b"123").expect("shrink file");

        assert_eq!(
            read_limited(file, &path, None, 8, capacity).expect("actual EOF"),
            b"123"
        );
    }

    #[test]
    fn source_io_rejects_limit_and_reservation_overflow() {
        let path = std::path::Path::new("overflow.pdf");
        let failure = read_limited(std::io::empty(), path, None, usize::MAX, 0)
            .expect_err("read sentinel overflow");
        assert_eq!(failure.operation, "validate size");
        assert!(failure.cause.contains("overflow sentinel"));

        let failure = source_buffer(usize::MAX, path, None).expect_err("reservation overflow");
        assert_eq!(failure.operation, "read");
        assert!(!failure.cause.is_empty());

        let failure = read_limited(std::io::empty(), path, None, usize::MAX - 1, usize::MAX - 1)
            .expect_err("bounded reader propagates reservation overflow");
        assert_eq!(failure.operation, "read");
        assert!(!failure.cause.is_empty());
    }

    #[test]
    fn actual_file_propagates_bound_and_reservation_overflow() {
        let path = std::path::Path::new("overflow.pdf");
        let file = tempfile::tempfile().expect("actual opened file");
        let failure = read_limited(file, path, None, usize::MAX, 0)
            .expect_err("actual file sentinel overflow");
        assert_eq!(failure.operation, "validate size");

        let file = tempfile::tempfile().expect("actual opened file");
        let failure = read_limited(file, path, None, usize::MAX - 1, usize::MAX - 1)
            .expect_err("actual file reservation overflow");
        assert_eq!(failure.operation, "read");
        assert!(!failure.cause.is_empty());
    }

    #[test]
    fn source_io_does_not_accept_partial_bytes_after_a_read_failure() {
        let path = std::path::Path::new("failed.pdf");
        let reader = std::io::Cursor::new(b"123").chain(FailingReader);
        let failure = read_limited(reader, path, None, 8, 3).expect_err("partial read failure");
        assert_eq!(failure.operation, "read");
        assert!(failure.cause.contains("deterministic read failure"));
    }

    #[test]
    fn file_url_rejects_a_relative_path() {
        let failure = file_url(
            std::path::Path::new("report.pdf"),
            BinaryDocumentFormat::Pdf,
        )
        .expect_err("relative file URL");

        assert_eq!(failure.operation, "canonicalize");
        assert!(failure.cause.contains("cannot be represented"));
    }

    #[test]
    fn source_io_covers_success_open_read_and_size_failures() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("report.pdf");
        std::fs::write(&path, b"%PDF-1.7").expect("fixture");

        assert!(
            file_url(&path, BinaryDocumentFormat::Pdf)
                .expect("file URL")
                .starts_with("file://")
        );
        assert_eq!(
            read_bounded(&path, Some(BinaryDocumentFormat::Pdf)).expect("bounded read"),
            b"%PDF-1.7"
        );
        assert_eq!(
            read_bounded_with_limit(&path, Some(BinaryDocumentFormat::Pdf), 4)
                .expect_err("size limit")
                .operation,
            "validate size"
        );
        assert_eq!(
            read_bounded(&directory.path().join("missing.pdf"), None)
                .expect_err("open failure")
                .operation,
            "read"
        );
        assert!(revision(b"document").starts_with("sha256:"));
    }

    #[test]
    fn source_io_covers_reader_and_remote_size_failures() {
        let path = std::path::Path::new("failed.pdf");
        let failure = read_limited(FailingReader, path, Some(BinaryDocumentFormat::Pdf), 8, 0)
            .expect_err("reader failure");
        assert_eq!(failure.operation, "read");
        assert!(failure.cause.contains("deterministic read failure"));

        assert!(enforce_remote_size("https://example.test/report.pdf", None, 0).is_ok());
        let failure = enforce_remote_size(
            "https://example.test/report.pdf",
            Some(BinaryDocumentFormat::Pdf),
            katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES + 1,
        )
        .expect_err("remote size limit");
        assert_eq!(failure.operation, "validate size");
        assert!(failure.cause.contains("limit is"));
    }
}
