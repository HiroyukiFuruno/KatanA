use crate::preview_pane::PreviewPane;
use katana_core::document::Document;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct DocumentFrameObservation {
    pub(super) format: String,
    pub(super) active_index: usize,
    pub(super) item_count: usize,
    pub(super) surface_kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct DocumentReceiptObservation {
    pub(super) uri: String,
    pub(super) generation: u64,
    pub(super) source_revision: String,
    pub(super) idle: bool,
    pub(super) frame: Option<DocumentFrameObservation>,
}

impl DocumentReceiptObservation {
    pub(super) fn capture(pane: &PreviewPane) -> Option<Self> {
        let (uri, generation) = pane.document_resource_identity()?;
        let source_revision = pane.document_source_revision_for_receipt()?.to_owned();
        let frame = pane.document_frame_state_for_test().map(
            |(format, active_index, item_count, surface_kind)| DocumentFrameObservation {
                format,
                active_index,
                item_count,
                surface_kind,
            },
        );
        Some(Self {
            uri,
            generation,
            source_revision,
            idle: pane.document_is_idle_for_test().unwrap_or(false),
            frame,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct HtmlLoadedSourceObservation {
    pub(super) preview_input_fnv1a64: String,
    pub(super) last_imported_disk_fnv1a64: Option<String>,
    pub(super) document_revision: u64,
    pub(super) saved_revision: u64,
    pub(super) dirty: bool,
    pub(super) is_reference: bool,
}

impl HtmlLoadedSourceObservation {
    pub(super) fn capture(document: Option<&Document>, preview_input_hash: u64) -> Option<Self> {
        let document = document.filter(|document| document.is_loaded)?;
        Some(Self {
            preview_input_fnv1a64: format!("{preview_input_hash:016x}"),
            last_imported_disk_fnv1a64: document
                .last_imported_disk_hash
                .map(|hash| format!("{hash:016x}")),
            document_revision: document.revision,
            saved_revision: document.saved_revision,
            dirty: document.is_dirty,
            is_reference: document.is_reference,
        })
    }
}

#[cfg(test)]
#[path = "resource_observation_tests.rs"]
mod tests;
