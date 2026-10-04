use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use skrifa::FontRef;

use super::resolver_selection::{PendingMatch, RequestKey};
use super::types::{FontFaceRequest, FontFaceResolutionDiagnostic};
use crate::debug_log::DebugLog;

pub(super) fn read_candidate(
    path: &Path,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, FontFaceResolutionDiagnostic> {
    DebugLog::write(
        "document_font_read",
        format_args!("phase=started path={}", path.display()),
    );
    let started = Instant::now();
    let result = super::font_file::read_candidate(path, cancelled);
    let (phase, bytes) = match &result {
        Ok(payload) => ("ready", payload.len()),
        Err(_) => ("failed", 0),
    };
    DebugLog::write(
        "document_font_read",
        format_args!(
            "phase={phase} path={} bytes={bytes} read_us={}",
            path.display(),
            started.elapsed().as_micros()
        ),
    );
    result
}

pub(super) fn selected_match(
    face: &FontRef<'_>,
    index: usize,
    path: &Path,
    requests: &BTreeMap<RequestKey, FontFaceRequest>,
) -> Vec<PendingMatch> {
    DebugLog::write(
        "document_font_metadata",
        format_args!("phase=started path={} face_index={index}", path.display()),
    );
    let started = Instant::now();
    let selected = super::resolver_selection::selected_match(face, index, requests);
    DebugLog::write(
        "document_font_metadata",
        format_args!(
            "phase=complete path={} face_index={index} selected={} metadata_us={}",
            path.display(),
            selected.len(),
            started.elapsed().as_micros()
        ),
    );
    selected
}
