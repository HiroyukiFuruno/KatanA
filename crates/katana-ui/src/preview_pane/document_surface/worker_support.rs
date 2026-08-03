use katana_core::document_source::BinaryDocumentFormat;
use katana_document_viewer::OfficeDocumentFormat;
use std::path::PathBuf;

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer};

pub(super) const INITIAL_VIEWPORT_WIDTH: u32 = 960;
pub(super) const INITIAL_VIEWPORT_HEIGHT: u32 = 640;
pub(super) const MIN_DOCUMENT_RENDER_SCALE: f32 = 0.25;
pub(super) const MAX_DOCUMENT_RENDER_SCALE: f32 = 4.0;

pub(super) fn office_format(source: &DocumentSurfaceSource) -> Option<OfficeDocumentFormat> {
    match source.format {
        BinaryDocumentFormat::Docx => Some(OfficeDocumentFormat::Docx),
        BinaryDocumentFormat::Xlsx => Some(OfficeDocumentFormat::Xlsx),
        BinaryDocumentFormat::Pptx => Some(OfficeDocumentFormat::Pptx),
        BinaryDocumentFormat::Pdf => None,
    }
}

pub(super) fn office_worker_executable(
    source: &DocumentSurfaceSource,
) -> Result<PathBuf, DocumentFailure> {
    if let Some(path) = std::env::var_os("KATANA_KDV_OFFICE_WORKER") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe().map_err(|error| {
        failure(
            source,
            "resolve worker",
            DocumentFailureLayer::KatanaHost,
            error,
        )
    })?;
    let parent = executable.parent().ok_or_else(|| {
        failure(
            source,
            "resolve worker",
            DocumentFailureLayer::KatanaHost,
            "application executable has no parent directory",
        )
    })?;
    let name = if cfg!(windows) {
        "kdv-office-worker.exe"
    } else {
        "kdv-office-worker"
    };
    Ok(parent.join(name))
}

pub(super) fn failure(
    source: &DocumentSurfaceSource,
    operation: &'static str,
    layer: DocumentFailureLayer,
    cause: impl std::fmt::Display,
) -> DocumentFailure {
    DocumentFailure::new(
        layer,
        operation,
        source.uri.clone(),
        Some(source.format),
        cause.to_string(),
    )
}
