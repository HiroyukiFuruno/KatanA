use katana_document_viewer::{DocumentViewerCommand, ViewerDocumentFormat};

use super::types::{DocumentFailure, DocumentSurface};
use super::worker::DocumentWorkerCommand;

impl DocumentSurface {
    pub(crate) fn frame_state_for_test(&self) -> Option<(String, usize, usize, String)> {
        let frame = self.frame.as_ref()?;
        Some((
            super::worker_support::format_extension(frame.format).to_owned(),
            frame.state.active_index,
            frame.state.item_count,
            format!("{:?}", frame.surface.kind()),
        ))
    }

    pub(crate) fn resource_counts_for_test(&self) -> (usize, usize) {
        (
            usize::from(self.frame.is_some()),
            usize::from(self.painter.texture.is_some()),
        )
    }

    pub(crate) fn pdf_outline_state(
        &self,
    ) -> Option<(Vec<katana_document_viewer::PdfOutlineItem>, usize)> {
        let frame = self.frame.as_ref()?;
        let outline_items = frame.surface.outline_items();
        if frame.format != ViewerDocumentFormat::Pdf || outline_items.is_empty() {
            return None;
        }
        Some((outline_items.to_vec(), frame.state.active_index))
    }

    pub(crate) fn next_for_test(&mut self) -> bool {
        let Some(frame) = &self.frame else {
            return false;
        };
        if frame.state.active_index.saturating_add(1) >= frame.state.item_count {
            return false;
        }
        self.queue(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next));
        true
    }

    pub(crate) fn failure_details_for_test(&self) -> Option<String> {
        self.failure.as_ref().map(DocumentFailure::details)
    }

    pub(crate) fn is_idle_for_test(&self) -> bool {
        self.frame.is_some()
            && self.failure.is_none()
            && !self.loading
            && !self.command_in_flight
            && self.pending_commands.is_empty()
    }
}
