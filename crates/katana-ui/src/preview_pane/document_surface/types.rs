use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::{
    DocumentSurfaceFrame, DocumentSurfaceHost, DocumentViewerState, DocumentViewport,
    ViewerCapabilities, ViewerDiagnostic,
};
use std::sync::mpsc::{Receiver, SyncSender};

use super::render_support::PendingDocumentCommands;
use super::source::DocumentSurfaceSource;
use super::worker::{DocumentWorkerCommand, DocumentWorkerEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentFailureLayer {
    SourceIntake,
    KdvWorker,
    KdvSurface,
    KatanaHost,
}

impl DocumentFailureLayer {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::SourceIntake => "source intake",
            Self::KdvWorker => "KDV worker",
            Self::KdvSurface => "KDV document surface",
            Self::KatanaHost => "KatanA host",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentFailure {
    pub layer: DocumentFailureLayer,
    pub operation: &'static str,
    pub document: String,
    pub format: Option<BinaryDocumentFormat>,
    pub cause: String,
}

impl DocumentFailure {
    pub(crate) fn new(
        layer: DocumentFailureLayer,
        operation: &'static str,
        document: String,
        format: Option<BinaryDocumentFormat>,
        cause: impl Into<String>,
    ) -> Self {
        Self {
            layer,
            operation,
            document,
            format,
            cause: cause.into(),
        }
    }

    pub(crate) fn intake(
        operation: &'static str,
        path: &std::path::Path,
        format: Option<BinaryDocumentFormat>,
        cause: impl Into<String>,
    ) -> Self {
        Self::new(
            DocumentFailureLayer::SourceIntake,
            operation,
            path.display().to_string(),
            format,
            cause,
        )
    }

    pub(crate) fn source_intake(
        operation: &'static str,
        document: impl Into<String>,
        format: Option<BinaryDocumentFormat>,
        cause: impl Into<String>,
    ) -> Self {
        Self::new(
            DocumentFailureLayer::SourceIntake,
            operation,
            document.into(),
            format,
            cause,
        )
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "{} could not display this {} document",
            self.layer.label(),
            self.format
                .map_or("binary", BinaryDocumentFormat::extension)
        )
    }

    pub(crate) fn details(&self) -> String {
        format!(
            "Layer: {}\nOperation: {}\nFormat: {}\nDocument: {}\nCause: {}",
            self.layer.label(),
            self.operation,
            self.format
                .map_or("unknown", BinaryDocumentFormat::extension),
            self.document,
            self.cause
        )
    }

    pub(crate) fn log(&self) {
        tracing::error!(
            layer = self.layer.label(),
            operation = self.operation,
            format = self
                .format
                .map_or("unknown", BinaryDocumentFormat::extension),
            document = self.document,
            cause = self.cause,
            "document viewer operation failed"
        );
    }
}

impl std::fmt::Display for DocumentFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.summary(), self.cause)
    }
}

#[derive(Debug)]
pub(super) struct DocumentFrame {
    pub surface: DocumentSurfaceFrame,
    pub state: DocumentViewerState,
    pub capabilities: ViewerCapabilities,
    pub diagnostics: Vec<ViewerDiagnostic>,
    pub format: BinaryDocumentFormat,
}

pub(crate) struct DocumentSurface {
    pub(super) generation: u64,
    pub(super) source: DocumentSurfaceSource,
    pub(super) command_tx: Option<SyncSender<DocumentWorkerCommand>>,
    pub(super) event_rx: Receiver<DocumentWorkerEvent>,
    pub(super) frame: Option<DocumentFrame>,
    pub(super) failure: Option<DocumentFailure>,
    pub(super) host: DocumentSurfaceHost,
    pub(super) loading: bool,
    pub(super) command_in_flight: bool,
    pub(super) pending_commands: PendingDocumentCommands,
    pub(super) viewport: Option<DocumentViewport>,
}

impl std::fmt::Debug for DocumentSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DocumentSurface")
            .field("generation", &self.generation)
            .field("source", &self.source)
            .field("loading", &self.loading)
            .field("failure", &self.failure)
            .finish_non_exhaustive()
    }
}

impl Drop for DocumentSurface {
    fn drop(&mut self) {
        self.command_tx.take();
    }
}
