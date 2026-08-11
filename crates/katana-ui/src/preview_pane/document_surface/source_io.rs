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
    let file = std::fs::File::open(path)
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    read_limited(file, path, format, limit)
}

fn read_limited(
    reader: impl Read,
    path: &Path,
    format: Option<BinaryDocumentFormat>,
    limit: usize,
) -> Result<Vec<u8>, DocumentFailure> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    enforce_size_limit(path, format, bytes.len(), limit)?;
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
        BinaryDocumentFormat, enforce_remote_size, file_url, read_bounded, read_bounded_with_limit,
        read_limited, revision,
    };

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("deterministic read failure"))
        }
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
        let failure = read_limited(FailingReader, path, Some(BinaryDocumentFormat::Pdf), 8)
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
