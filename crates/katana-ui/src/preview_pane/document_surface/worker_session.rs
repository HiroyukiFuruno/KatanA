use katana_document_viewer::{
    DocumentSession, DocumentSessionConfig, DocumentViewport, OfficeWorkerConfig,
};

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer};
use super::worker_support::{
    INITIAL_VIEWPORT_HEIGHT, INITIAL_VIEWPORT_WIDTH, failure, office_worker_executable,
};

pub(super) fn open_session(
    source: &mut DocumentSurfaceSource,
) -> Result<DocumentSession, DocumentFailure> {
    let viewer_source = source.take_viewer_source()?;
    let viewport = DocumentViewport::new(INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);
    let config = session_config(source, viewport)?;
    DocumentSession::open(viewer_source, config)
        .map_err(|error| failure(source, "open", DocumentFailureLayer::KdvWorker, error))
}

fn session_config(
    source: &DocumentSurfaceSource,
    viewport: DocumentViewport,
) -> Result<DocumentSessionConfig, DocumentFailure> {
    let config = DocumentSessionConfig::new(viewport);
    if source.format == katana_core::document_source::BinaryDocumentFormat::Pdf {
        return Ok(config);
    }
    Ok(config.office_worker(OfficeWorkerConfig::new(office_worker_executable(source)?)))
}
