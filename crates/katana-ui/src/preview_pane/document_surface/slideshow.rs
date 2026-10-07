use eframe::egui;
use katana_document_viewer::DocumentViewerCommand;

use super::painter::paint_document_frame;
use super::render_support::show_failure;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker::DocumentWorkerCommand;

impl DocumentSurface {
    pub(crate) fn is_pptx(&self) -> bool {
        self.source.format == katana_core::document_source::BinaryDocumentFormat::Pptx
    }

    pub(crate) fn slideshow_state(&self) -> Option<(usize, usize)> {
        self.frame
            .as_ref()
            .map(|frame| (frame.state.active_index, frame.state.item_count))
    }

    pub(crate) fn slideshow_step(&mut self, next: bool) {
        let Some(frame) = &self.frame else { return };
        let index = frame.state.active_index;
        let count = frame.state.item_count;
        let command = if next {
            (index + 1 < count).then_some(DocumentViewerCommand::Next)
        } else {
            (index > 0).then_some(DocumentViewerCommand::Previous)
        };
        if let Some(command) = command {
            self.queue(DocumentWorkerCommand::Viewer(command));
        }
    }

    pub(crate) fn slideshow_jump_to(&mut self, index: usize) {
        let Some(frame) = &self.frame else { return };
        if index < frame.state.item_count && index != frame.state.active_index {
            self.queue(DocumentWorkerCommand::Viewer(
                DocumentViewerCommand::JumpTo(index),
            ));
        }
    }

    pub(crate) fn show_slideshow(&mut self, ui: &mut egui::Ui) {
        self.poll(ui.ctx());
        if let Some(failure) = self.failure.clone() {
            show_failure(ui, &failure);
            return;
        }
        let Some(frame) = self.frame.take() else {
            ui.centered_and_justified(|ui| ui.spinner());
            return;
        };
        match paint_document_frame(
            &mut self.painter,
            ui,
            &frame.surface,
            self.generation,
            &self.border_cache,
        ) {
            Ok(commands) => {
                for command in commands {
                    self.queue_surface(command);
                }
                self.frame = Some(frame);
            }
            Err(error) => {
                let failure = DocumentFailure::new(
                    DocumentFailureLayer::KatanaHost,
                    "paint slideshow document",
                    self.source.uri.clone(),
                    Some(self.source.format),
                    error.to_string(),
                );
                failure.log();
                self.failure = Some(failure);
                self.frame = Some(frame);
            }
        }
    }
}
