use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeDocumentSource, ViewerSource,
    ViewerSourceIdentity,
};
use std::path::{Path, PathBuf};

use super::super::cancellable_read::ReadCancellation;
use super::source_io::{enforce_remote_size, revision};
use super::types::{DocumentFailure, DocumentFailureLayer};
use super::worker_memory::IntakeMemoryLease;

#[path = "source_local.rs"]
mod local;

#[derive(Debug, Clone)]
pub(crate) struct DocumentSurfaceSource {
    pub uri: String,
    pub format: BinaryDocumentFormat,
    pub mime: String,
    pub revision: String,
    bytes: Vec<u8>,
    /* WHY: bytesとその複製・送信結果が解放された後にだけ読込分の返却待機を解除する。 */
    _intake_memory: Option<std::sync::Arc<IntakeMemoryLease<'static>>>,
}

impl DocumentSurfaceSource {
    #[cfg(test)]
    pub(crate) fn local(path: &Path) -> Result<Self, DocumentFailure> {
        Self::local_with_cancellation(path, || false)
    }

    pub(super) fn local_with_cancellation(
        path: &Path,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, DocumentFailure> {
        let cancellation = ReadCancellation::new(cancelled);
        local::SourceLocalOps::load(path, &cancellation)
    }

    pub(crate) fn remote(
        uri: String,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<Self, DocumentFailure> {
        let started_at = std::time::Instant::now();
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
        let source = Self {
            uri,
            format,
            mime: format.mime().to_owned(),
            revision: revision(&bytes),
            bytes,
            _intake_memory: None,
        };
        super::debug_log::DebugLog::write(
            "document_source_intake",
            format_args!(
                "kind=remote format={} bytes={} elapsed_ms={} uri={}",
                source.format.extension(),
                source.bytes.len(),
                started_at.elapsed().as_millis(),
                source.uri
            ),
        );
        Ok(source)
    }

    pub(super) fn descriptor(&self) -> Self {
        Self {
            uri: self.uri.clone(),
            format: self.format,
            mime: self.mime.clone(),
            revision: self.revision.clone(),
            bytes: Vec::new(),
            _intake_memory: None,
        }
    }

    pub(super) const fn byte_len(&self) -> usize {
        self.bytes.len()
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

#[cfg(test)]
#[path = "source_memory_tests.rs"]
mod memory_tests;
