use katana_document_viewer::{
    DocumentSession, DocumentSessionConfig, DocumentViewport, OfficeWorkerConfig,
};

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer};
use super::worker_cancellation::DocumentWorkerCancellation;
use super::worker_support::{
    INITIAL_VIEWPORT_HEIGHT, INITIAL_VIEWPORT_WIDTH, failure, office_worker_executable,
};

pub(super) fn open_session(
    source: &mut DocumentSurfaceSource,
    cancellation: &DocumentWorkerCancellation,
) -> Result<Option<DocumentSession>, DocumentFailure> {
    if cancellation.is_cancelled() {
        return Ok(None);
    }
    let viewer_source = source.take_viewer_source()?;
    super::debug_log::DebugLog::write(
        "document_viewer_source_ready",
        format_args!("format={}", source.format.extension()),
    );
    let viewport = DocumentViewport::new(INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);
    let config = session_config(source, viewport)?;
    /* WHY: 検証中の取消を反映してから、中断APIのないnative openへ進む。 */
    if cancellation.is_cancelled() {
        return Ok(None);
    }
    super::debug_log::DebugLog::write(
        "document_session_native_open_begin",
        format_args!("format={}", source.format.extension()),
    );
    DocumentSession::open(viewer_source, config)
        .map(Some)
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
