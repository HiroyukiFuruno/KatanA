use std::path::Path;
use thiserror::Error;

const PDF_MAGIC: &[u8] = b"%PDF-";
const ZIP_LOCAL_MAGIC: &[u8] = &[0x50, 0x4b, 0x03, 0x04];
const ZIP_EMPTY_MAGIC: &[u8] = &[0x50, 0x4b, 0x05, 0x06];
const ZIP_SPANNED_MAGIC: &[u8] = &[0x50, 0x4b, 0x07, 0x08];

pub const MAX_BINARY_DOCUMENT_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryDocumentFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
}

impl BinaryDocumentFormat {
    pub fn detect(
        path: &Path,
        content_type: Option<&str>,
        bytes: &[u8],
    ) -> Result<Self, DocumentSourceError> {
        let extension_format = Self::from_path(path);
        let mime_format = Self::from_content_type(content_type)?;
        if let (Some(extension), Some(mime)) = (extension_format, mime_format)
            && extension != mime
        {
            return Err(DocumentSourceError::MetadataMismatch { extension, mime });
        }
        let format = mime_format
            .or(extension_format)
            .ok_or(DocumentSourceError::UnsupportedFormat)?;
        format.validate_signature(bytes)?;
        Ok(format)
    }

    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        match extension.to_ascii_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "xlsx" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            _ => None,
        }
    }

    #[must_use]
    pub const fn mime(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
        }
    }

    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
        }
    }

    fn from_content_type(content_type: Option<&str>) -> Result<Option<Self>, DocumentSourceError> {
        let Some(content_type) = content_type else {
            return Ok(None);
        };
        let normalized = content_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if normalized.is_empty() || is_generic_binary_mime(&normalized) {
            return Ok(None);
        }
        document_format_for_mime(&normalized)
            .map(Some)
            .ok_or(DocumentSourceError::UnsupportedContentType(normalized))
    }

    fn validate_signature(self, bytes: &[u8]) -> Result<(), DocumentSourceError> {
        let valid = match self {
            Self::Pdf => bytes.starts_with(PDF_MAGIC),
            Self::Docx | Self::Xlsx | Self::Pptx => has_zip_signature(bytes),
        };
        if valid {
            return Ok(());
        }
        Err(DocumentSourceError::SignatureMismatch(self))
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DocumentSourceError {
    #[error("document format is not supported")]
    UnsupportedFormat,
    #[error("document content type is not supported: {0}")]
    UnsupportedContentType(String),
    #[error("document extension is {extension:?}, but content type identifies {mime:?}")]
    MetadataMismatch {
        extension: BinaryDocumentFormat,
        mime: BinaryDocumentFormat,
    },
    #[error("document bytes do not match the {0:?} container signature")]
    SignatureMismatch(BinaryDocumentFormat),
}

fn is_generic_binary_mime(value: &str) -> bool {
    matches!(value, "application/octet-stream" | "binary/octet-stream")
}

fn document_format_for_mime(value: &str) -> Option<BinaryDocumentFormat> {
    match value {
        "application/pdf" => Some(BinaryDocumentFormat::Pdf),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            Some(BinaryDocumentFormat::Docx)
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            Some(BinaryDocumentFormat::Xlsx)
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            Some(BinaryDocumentFormat::Pptx)
        }
        _ => None,
    }
}

fn has_zip_signature(bytes: &[u8]) -> bool {
    bytes.starts_with(ZIP_LOCAL_MAGIC)
        || bytes.starts_with(ZIP_EMPTY_MAGIC)
        || bytes.starts_with(ZIP_SPANNED_MAGIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZIP_BYTES: &[u8] = &[0x50, 0x4b, 0x03, 0x04, 1];

    #[test]
    fn detects_each_supported_format_from_matching_metadata_and_signature() {
        let cases = [
            (BinaryDocumentFormat::Pdf, b"%PDF-1.7".as_slice()),
            (BinaryDocumentFormat::Docx, ZIP_BYTES),
            (BinaryDocumentFormat::Xlsx, ZIP_BYTES),
            (BinaryDocumentFormat::Pptx, ZIP_BYTES),
        ];
        for (format, bytes) in cases {
            let path = format!("document.{}", format.extension().to_uppercase());
            assert_eq!(
                Ok(format),
                BinaryDocumentFormat::detect(Path::new(&path), Some(format.mime()), bytes)
            );
            assert_eq!(
                Some(format),
                BinaryDocumentFormat::from_path(Path::new(&path))
            );
        }
    }

    #[test]
    fn accepts_mime_parameters_generic_binary_and_mime_only_sources() {
        assert_eq!(
            Ok(BinaryDocumentFormat::Pdf),
            BinaryDocumentFormat::detect(
                Path::new("download"),
                Some("Application/PDF; charset=binary"),
                b"%PDF-1.7",
            )
        );
        assert_eq!(
            Ok(BinaryDocumentFormat::Docx),
            BinaryDocumentFormat::detect(
                Path::new("document.docx"),
                Some("application/octet-stream"),
                ZIP_BYTES,
            )
        );
        assert_eq!(
            Ok(BinaryDocumentFormat::Xlsx),
            BinaryDocumentFormat::detect(Path::new("sheet.xlsx"), Some(""), ZIP_BYTES)
        );
    }

    #[test]
    fn rejects_unsupported_conflicting_and_malformed_sources() {
        assert_eq!(
            Err(DocumentSourceError::UnsupportedFormat),
            BinaryDocumentFormat::detect(Path::new("readme.md"), None, b"text")
        );
        assert_eq!(
            Err(DocumentSourceError::UnsupportedContentType(
                "text/html".to_owned()
            )),
            BinaryDocumentFormat::detect(Path::new("document.pdf"), Some("text/html"), b"html")
        );
        assert_eq!(
            Err(DocumentSourceError::MetadataMismatch {
                extension: BinaryDocumentFormat::Pdf,
                mime: BinaryDocumentFormat::Docx,
            }),
            BinaryDocumentFormat::detect(
                Path::new("document.pdf"),
                Some(BinaryDocumentFormat::Docx.mime()),
                ZIP_BYTES,
            )
        );
        for format in [
            BinaryDocumentFormat::Pdf,
            BinaryDocumentFormat::Docx,
            BinaryDocumentFormat::Xlsx,
            BinaryDocumentFormat::Pptx,
        ] {
            let path = format!("document.{}", format.extension());
            assert_eq!(
                Err(DocumentSourceError::SignatureMismatch(format)),
                BinaryDocumentFormat::detect(Path::new(&path), None, b"invalid")
            );
        }
    }

    #[test]
    fn accepts_all_zip_container_signatures_and_binary_mime_alias() {
        for signature in [ZIP_LOCAL_MAGIC, ZIP_EMPTY_MAGIC, ZIP_SPANNED_MAGIC] {
            assert_eq!(
                Ok(BinaryDocumentFormat::Pptx),
                BinaryDocumentFormat::detect(
                    Path::new("slides.pptx"),
                    Some("binary/octet-stream"),
                    signature,
                )
            );
        }
    }
}
