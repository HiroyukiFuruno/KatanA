use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeDocumentSource, OfficeStaticViewerSession,
    OfficeWorkerConfig, PdfViewerSession, SpreadsheetViewerSession, ViewerSourceIdentity,
};
use std::path::PathBuf;

use super::paged_runtime::PagedRuntime;
use super::source::DocumentSurfaceSource;
use super::spreadsheet_runtime::SpreadsheetRuntime;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentFrame};
use super::worker::DocumentWorkerCommand;
use super::worker_support::{failure, office_format, office_worker_executable};

pub(super) enum DocumentRuntime {
    Paged(Box<PagedRuntime>),
    Spreadsheet(Box<SpreadsheetRuntime>),
}

impl DocumentRuntime {
    pub(super) fn open(mut source: DocumentSurfaceSource) -> Result<Self, DocumentFailure> {
        let bytes = source.take_bytes()?;
        validate_source(&source, &bytes)?;
        let identity = ViewerSourceIdentity::new(source.uri.clone(), source.revision.clone());
        match source.format {
            BinaryDocumentFormat::Pdf => Self::open_pdf(&source, identity, bytes),
            BinaryDocumentFormat::Docx | BinaryDocumentFormat::Pptx => {
                Self::open_static_office(&source, identity, bytes)
            }
            BinaryDocumentFormat::Xlsx => Self::open_spreadsheet(&source, identity, bytes),
        }
    }

    pub(super) fn apply(&mut self, command: DocumentWorkerCommand) -> Result<(), DocumentFailure> {
        match self {
            Self::Paged(runtime) => runtime.apply(command),
            Self::Spreadsheet(runtime) => runtime.apply(command),
        }
    }

    pub(super) fn frame(&mut self) -> Result<DocumentFrame, DocumentFailure> {
        match self {
            Self::Paged(runtime) => runtime.frame(),
            Self::Spreadsheet(runtime) => runtime.frame(),
        }
    }

    fn open_pdf(
        source: &DocumentSurfaceSource,
        identity: ViewerSourceIdentity,
        bytes: Vec<u8>,
    ) -> Result<Self, DocumentFailure> {
        let pdf = PdfViewerSession::open(BinaryDocumentSource::new(
            identity,
            source.mime.clone(),
            bytes,
        ))
        .map_err(|error| failure(source, "open", DocumentFailureLayer::KdvWorker, error))?;
        Ok(Self::Paged(Box::new(PagedRuntime::from_pdf(source, pdf))))
    }

    fn open_static_office(
        source: &DocumentSurfaceSource,
        identity: ViewerSourceIdentity,
        bytes: Vec<u8>,
    ) -> Result<Self, DocumentFailure> {
        let format = office_format(source).ok_or_else(|| {
            failure(
                source,
                "route",
                DocumentFailureLayer::KatanaHost,
                "PDF cannot be routed to an Office session",
            )
        })?;
        let office_source = OfficeDocumentSource::new(identity, format, source.mime.clone(), bytes);
        let session = OfficeStaticViewerSession::open(
            office_source,
            OfficeWorkerConfig::new(office_worker_executable(source)?),
        )
        .map_err(|error| failure(source, "open", DocumentFailureLayer::KdvWorker, error))?;
        Ok(Self::Paged(Box::new(PagedRuntime::from_office(
            source, session,
        ))))
    }

    fn open_spreadsheet(
        source: &DocumentSurfaceSource,
        identity: ViewerSourceIdentity,
        bytes: Vec<u8>,
    ) -> Result<Self, DocumentFailure> {
        let office_source = OfficeDocumentSource::new(
            identity,
            OfficeDocumentFormat::Xlsx,
            source.mime.clone(),
            bytes,
        );
        let session = SpreadsheetViewerSession::open(
            office_source,
            OfficeWorkerConfig::new(office_worker_executable(source)?),
        )
        .map_err(|error| failure(source, "open", DocumentFailureLayer::KdvWorker, error))?;
        Ok(Self::Spreadsheet(Box::new(SpreadsheetRuntime::new(
            source, session,
        )?)))
    }
}

fn validate_source(source: &DocumentSurfaceSource, bytes: &[u8]) -> Result<(), DocumentFailure> {
    let synthetic_path = PathBuf::from(format!("document.{}", source.format.extension()));
    BinaryDocumentFormat::detect(&synthetic_path, Some(&source.mime), bytes).map_err(|error| {
        failure(
            source,
            "validate",
            DocumentFailureLayer::SourceIntake,
            error,
        )
    })?;
    Ok(())
}
