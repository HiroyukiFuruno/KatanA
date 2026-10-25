use eframe::egui;
use katana_document_viewer::{DocumentFrame, DocumentViewerCommand, ViewerDocumentFormat};

use super::types::DocumentSurface;
use super::worker::DocumentWorkerCommand;

pub(super) fn show(surface: &mut DocumentSurface, ui: &mut egui::Ui, frame: &DocumentFrame) {
    if frame.surface.grid().is_some()
        || !matches!(
            frame.format,
            ViewerDocumentFormat::Pdf | ViewerDocumentFormat::Pptx
        )
    {
        return;
    }

    let response = ui.interact(
        ui.available_rect_before_wrap(),
        egui::Id::new(("document-page-navigation", surface.generation)),
        egui::Sense::focusable_noninteractive(),
    );
    if response.hovered() && ui.input(|input| input.pointer.any_click()) {
        response.request_focus();
    }
    if let Some(command) = navigation_command(ui, &response, frame) {
        surface.queue(DocumentWorkerCommand::Viewer(command));
    }
}

fn navigation_command(
    ui: &egui::Ui,
    response: &egui::Response,
    frame: &DocumentFrame,
) -> Option<DocumentViewerCommand> {
    if !response.has_focus()
        || !matches!(
            frame.format,
            ViewerDocumentFormat::Pdf | ViewerDocumentFormat::Pptx
        )
        || ui.input(|input| input.modifiers != egui::Modifiers::default())
    {
        return None;
    }

    ui.input(|input| {
        if input.key_pressed(egui::Key::ArrowLeft) && frame.state.active_index > 0 {
            Some(DocumentViewerCommand::Previous)
        } else if input.key_pressed(egui::Key::ArrowRight)
            && frame.state.active_index.saturating_add(1) < frame.state.item_count
        {
            Some(DocumentViewerCommand::Next)
        } else {
            None
        }
    })
}

#[cfg(test)]
#[path = "render_keyboard_tests.rs"]
mod keyboard_tests;
