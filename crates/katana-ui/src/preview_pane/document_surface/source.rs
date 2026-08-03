use katana_core::document_source::BinaryDocumentFormat;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::types::{DocumentFailure, DocumentFailureLayer};

const HEX_NIBBLE_BITS: u8 = 4;
const HEX_NIBBLE_MASK: u8 = 0x0f;

#[derive(Debug, Clone)]
pub(crate) struct DocumentSurfaceSource {
    pub uri: String,
    pub format: BinaryDocumentFormat,
    pub mime: String,
    pub revision: String,
    bytes: Vec<u8>,
}

impl DocumentSurfaceSource {
    pub(crate) fn local(path: &Path) -> Result<Self, DocumentFailure> {
        let canonical = path.canonicalize().map_err(|error| {
            DocumentFailure::intake("canonicalize", path, None, error.to_string())
        })?;
        let format = BinaryDocumentFormat::from_path(&canonical).ok_or_else(|| {
            DocumentFailure::intake(
                "classify",
                &canonical,
                None,
                "document extension is not supported",
            )
        })?;
        let bytes = read_bounded(&canonical, Some(format))?;
        BinaryDocumentFormat::detect(&canonical, Some(format.mime()), &bytes).map_err(|error| {
            DocumentFailure::intake("validate", &canonical, Some(format), error.to_string())
        })?;
        let uri = file_url(&canonical, format)?;
        Ok(Self {
            uri,
            format,
            mime: format.mime().to_owned(),
            revision: revision(&bytes),
            bytes,
        })
    }

    pub(crate) fn remote(
        uri: String,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<Self, DocumentFailure> {
        let path = url::Url::parse(&uri)
            .ok()
            .map(|url| PathBuf::from(url.path()))
            .unwrap_or_default();
        let format_hint = BinaryDocumentFormat::from_path(&path);
        enforce_remote_size(&uri, format_hint, bytes.len())?;
        let format =
            BinaryDocumentFormat::detect(&path, content_type, &bytes).map_err(|error| {
                DocumentFailure::source_intake(
                    "classify",
                    uri.clone(),
                    format_hint,
                    error.to_string(),
                )
            })?;
        Ok(Self {
            uri,
            format,
            mime: format.mime().to_owned(),
            revision: revision(&bytes),
            bytes,
        })
    }

    pub(super) fn descriptor(&self) -> Self {
        Self {
            uri: self.uri.clone(),
            format: self.format,
            mime: self.mime.clone(),
            revision: self.revision.clone(),
            bytes: Vec::new(),
        }
    }

    pub(super) fn take_bytes(&mut self) -> Result<Vec<u8>, DocumentFailure> {
        if self.bytes.is_empty() {
            return Err(DocumentFailure::new(
                DocumentFailureLayer::SourceIntake,
                "read",
                self.uri.clone(),
                Some(self.format),
                "document source bytes are unavailable",
            ));
        }
        Ok(std::mem::take(&mut self.bytes))
    }
}

impl PartialEq for DocumentSurfaceSource {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
            && self.format == other.format
            && self.mime == other.mime
            && self.revision == other.revision
    }
}

impl Eq for DocumentSurfaceSource {}

fn revision(bytes: &[u8]) -> String {
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

fn file_url(path: &Path, format: BinaryDocumentFormat) -> Result<String, DocumentFailure> {
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

fn read_bounded(
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
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| DocumentFailure::intake("read", path, format, error.to_string()))?;
    enforce_size_limit(path, format, bytes.len(), limit)?;
    Ok(bytes)
}

fn enforce_remote_size(
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
    use super::{BinaryDocumentFormat, file_url};

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
}
