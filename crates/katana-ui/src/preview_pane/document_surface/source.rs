use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeDocumentSource, ViewerSource,
    ViewerSourceIdentity,
};
use std::path::{Path, PathBuf};

use super::source_io::{enforce_remote_size, file_url, read_bounded, revision};
use super::types::{DocumentFailure, DocumentFailureLayer};

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

    pub(super) fn take_viewer_source(&mut self) -> Result<ViewerSource, DocumentFailure> {
        let identity = ViewerSourceIdentity::new(self.uri.clone(), self.revision.clone());
        let bytes = self.take_bytes()?;
        Ok(match self.format {
            BinaryDocumentFormat::Pdf => ViewerSource::Pdf(BinaryDocumentSource::new(
                identity,
                self.mime.clone(),
                bytes,
            )),
            BinaryDocumentFormat::Docx => {
                self.office_source(identity, bytes, OfficeDocumentFormat::Docx)
            }
            BinaryDocumentFormat::Xlsx => {
                self.office_source(identity, bytes, OfficeDocumentFormat::Xlsx)
            }
            BinaryDocumentFormat::Pptx => {
                self.office_source(identity, bytes, OfficeDocumentFormat::Pptx)
            }
        })
    }

    fn office_source(
        &self,
        identity: ViewerSourceIdentity,
        bytes: Vec<u8>,
        format: OfficeDocumentFormat,
    ) -> ViewerSource {
        ViewerSource::Office(OfficeDocumentSource::new(
            identity,
            format,
            self.mime.clone(),
            bytes,
        ))
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
