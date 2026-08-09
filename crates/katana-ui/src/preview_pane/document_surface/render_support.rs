use eframe::egui;
use katana_document_viewer::{DocumentGridCommand, DocumentSurfaceCommand, DocumentViewerCommand};
use std::collections::VecDeque;

use super::types::DocumentFailure;
use super::worker::DocumentWorkerCommand;

const MAX_PENDING_USER_COMMANDS: usize = 16;

#[derive(Debug, Default)]
pub(super) struct PendingDocumentCommands {
    user: VecDeque<DocumentWorkerCommand>,
    resize: Option<DocumentWorkerCommand>,
}

impl PendingDocumentCommands {
    pub(super) fn push(&mut self, command: DocumentWorkerCommand) {
        match command {
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Resize(_)) => {
                self.resize = Some(command);
            }
            _ => {
                if let Some(last) = self.user.back_mut()
                    && coalesces(last, &command)
                {
                    *last = command;
                    return;
                }
                if self.user.len() == MAX_PENDING_USER_COMMANDS {
                    self.user.pop_front();
                }
                self.user.push_back(command);
            }
        }
    }

    pub(super) fn take_next(&mut self) -> Option<DocumentWorkerCommand> {
        self.user.pop_front().or_else(|| self.resize.take())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.user.is_empty() && self.resize.is_none()
    }

    pub(super) fn clear(&mut self) {
        self.user.clear();
        self.resize = None;
    }
}

fn coalesces(previous: &DocumentWorkerCommand, next: &DocumentWorkerCommand) -> bool {
    matches!(
        (previous, next),
        (
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(
                DocumentGridCommand::ScrollTo { .. }
            )),
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(
                DocumentGridCommand::ScrollTo { .. }
            ))
        ) | (
            DocumentWorkerCommand::Viewer(DocumentViewerCommand::SetZoom(_)),
            DocumentWorkerCommand::Viewer(DocumentViewerCommand::SetZoom(_))
        ) | (
            DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(_)),
            DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(_))
        )
    )
}

pub(super) fn show_failure(ui: &mut egui::Ui, failure: &DocumentFailure) {
    let messages = crate::i18n::I18nOps::get();
    ui.vertical(|ui| {
        ui.colored_label(ui.visuals().error_fg_color, failure.summary());
        egui::CollapsingHeader::new(&messages.preview.document_controller.error_details)
            .default_open(true)
            .show(ui, |ui| {
                ui.monospace(failure.details());
            });
    });
}

pub(super) fn show_diagnostics(ui: &mut egui::Ui, frame: &katana_document_viewer::DocumentFrame) {
    if frame.diagnostics.is_empty() {
        return;
    }
    let messages = crate::i18n::I18nOps::get();
    let count = frame.diagnostics.len().to_string();
    let label = crate::i18n::I18nOps::tf(
        &messages.preview.document_controller.rendering_notes,
        &[("count", &count)],
    );
    egui::CollapsingHeader::new(label).show(ui, |ui| {
        for diagnostic in &frame.diagnostics {
            ui.label(&diagnostic.message);
        }
    });
}
