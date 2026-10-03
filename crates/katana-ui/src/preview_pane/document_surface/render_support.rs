use eframe::egui;
use katana_document_viewer::{
    DocumentGridCommand, DocumentSurfaceCommand, DocumentViewerCommand, SpreadsheetFilterCommand,
};
use std::collections::VecDeque;

use super::types::DocumentFailure;
use super::worker::DocumentWorkerCommand;

const MAX_PENDING_USER_COMMANDS: usize = 16;

#[cfg(test)]
#[path = "render_support_queue_tests.rs"]
mod queue_tests;

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
            _ => self.push_user(command),
        }
    }

    fn push_user(&mut self, command: DocumentWorkerCommand) {
        if is_candidates(&command) {
            self.user.retain(|pending| !is_candidates(pending));
        } else {
            if let Some(last) = self.user.back_mut()
                && coalesces(last, &command)
            {
                *last = command;
                return;
            }
            self.evict_oldest_user_if_full();
        }
        self.user.push_back(command);
    }

    fn evict_oldest_user_if_full(&mut self) {
        let user_count = self
            .user
            .iter()
            .filter(|pending| !is_candidates(pending))
            .count();
        if user_count == MAX_PENDING_USER_COMMANDS
            && let Some(index) = self.user.iter().position(|pending| !is_candidates(pending))
        {
            self.user.remove(index);
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

fn is_candidates(command: &DocumentWorkerCommand) -> bool {
    matches!(
        command,
        DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::Candidates { .. })
    )
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
