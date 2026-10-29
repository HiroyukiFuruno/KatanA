use eframe::egui;
use katana_document_viewer::{DocumentFitMode, DocumentSurfaceCommand, DocumentViewerCommand};

use super::controls::show_controls;
use super::controls_sheet_tabs::{sheet_tab_rail_height, show_sheet_tabs};
use super::painter::paint_document_frame;
use super::render_support::{show_diagnostics, show_failure};
use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker::DocumentWorkerCommand;

const DOCUMENT_DIAGNOSTIC_GAP: f32 = 4.0;
const SHEET_RAIL_HORIZONTAL_MARGIN: i8 = 6;
const SHEET_RAIL_VERTICAL_MARGIN: i8 = 4;

impl DocumentSurface {
    pub(crate) fn source(&self) -> &DocumentSurfaceSource {
        &self.source
    }

    pub(crate) fn jump_to_item(&mut self, index: usize) {
        self.queue(DocumentWorkerCommand::Viewer(
            DocumentViewerCommand::JumpTo(index),
        ));
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        self.poll(ui.ctx());
        if let Some(failure) = self.failure.clone() {
            show_failure(ui, &failure);
            return;
        }
        let Some(frame) = self.frame.take() else {
            let loading = crate::widgets::LoadingPresentation::new(
                crate::widgets::LoadingState::Pending,
                crate::widgets::LoadingLabel::RenderingKind(self.source.format.extension()),
            );
            crate::widgets::LoadingIndicator::show_centered(ui, &loading);
            return;
        };

        show_controls(self, ui, &frame);
        /* WHY: Place diagnostics before allocating the grid so it cannot overlap the bottom tab input region. */
        show_diagnostics(ui, &frame);
        ui.add_space(DOCUMENT_DIAGNOSTIC_GAP);
        super::page_navigation::show(self, ui, &frame);
        let sheet_tab_height = sheet_tab_rail_height(frame.format);
        if sheet_tab_height > 0.0 {
            let rail_frame = egui::Frame::side_top_panel(ui.style()).inner_margin(
                egui::Margin::symmetric(SHEET_RAIL_HORIZONTAL_MARGIN, SHEET_RAIL_VERTICAL_MARGIN),
            );
            egui::Panel::bottom(egui::Id::new(("document-sheet-tabs-rail", self.generation)))
                .resizable(false)
                .exact_size(sheet_tab_height)
                .frame(rail_frame)
                .show_inside(ui, |ui| show_sheet_tabs(self, ui, &frame));
        }
        let painted = if frame.surface.grid().is_some() {
            super::painter_grid::paint_with_filters_and_fonts(
                ui,
                &frame.surface,
                &mut self.filter_ui,
                &self.border_cache,
                self.fonts.lease(),
            )
        } else {
            paint_document_frame(
                &mut self.painter,
                ui,
                &frame.surface,
                self.generation,
                &self.border_cache,
            )
            .map(|commands| (commands, Vec::new()))
        };
        let (commands, filter_commands) = match painted {
            Ok(value) => value,
            Err(error) => {
                let failure = DocumentFailure::new(
                    DocumentFailureLayer::KatanaHost,
                    "paint spreadsheet borders",
                    self.source.uri.clone(),
                    Some(self.source.format),
                    error.to_string(),
                );
                failure.log();
                self.failure = Some(failure);
                self.frame = Some(frame);
                return;
            }
        };
        for command in commands {
            self.queue_surface(command);
        }
        for command in filter_commands {
            self.queue(DocumentWorkerCommand::SpreadsheetFilter(command));
        }
        self.frame = Some(frame);
    }

    pub(super) fn queue(&mut self, command: DocumentWorkerCommand) {
        if self.command_in_flight {
            tracing::debug!(
                generation = self.generation,
                ?command,
                "queued document command behind in-flight work"
            );
            if !self.pending_commands.push(command) {
                self.fail_command_enqueue();
            }
            return;
        }
        self.send(command);
    }

    pub(super) fn queue_surface(&mut self, command: DocumentSurfaceCommand) {
        if let DocumentSurfaceCommand::Resize(viewport) = command
            && !replace_viewport_if_changed(&mut self.viewport, viewport)
        {
            return;
        }
        self.queue(DocumentWorkerCommand::Surface(command));
    }

    pub(super) fn set_fit(&mut self, fit: DocumentFitMode) {
        self.queue(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(
            fit,
        )));
    }
}

pub(super) fn replace_viewport_if_changed(
    current: &mut Option<katana_document_viewer::DocumentViewport>,
    next: katana_document_viewer::DocumentViewport,
) -> bool {
    if *current == Some(next) {
        return false;
    }
    *current = Some(next);
    true
}
