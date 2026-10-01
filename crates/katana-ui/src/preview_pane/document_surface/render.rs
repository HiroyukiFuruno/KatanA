use eframe::egui;
use katana_document_viewer::{DocumentFitMode, DocumentSurfaceCommand, DocumentViewerCommand};
use std::sync::atomic::{AtomicU64, Ordering};

use super::controls::show_controls;
use super::controls_sheet_tabs::{sheet_tab_rail_height, show_sheet_tabs};
use super::painter::paint_document_frame;
use super::render_support::{PendingDocumentCommands, show_diagnostics, show_failure};
use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker::{self, DocumentWorkerCommand};

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

impl DocumentSurface {
    pub(crate) fn start(source: DocumentSurfaceSource, ctx: &egui::Context) -> Self {
        let started_at = std::time::Instant::now();
        let generation = NEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
        let (command_tx, command_rx) = std::sync::mpsc::sync_channel(1);
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let display_source = source.descriptor();
        super::debug_log::DebugLog::write(
            "document_worker_handoff",
            format_args!(
                "generation={} format={} bytes={} uri={}",
                generation,
                source.format.extension(),
                source.byte_len(),
                source.uri
            ),
        );
        let worker_source = source;
        let repaint = ctx.clone();
        let spawn = std::thread::Builder::new()
            .name(format!("katana-document-{generation}"))
            .spawn(move || worker::run(generation, worker_source, command_rx, event_tx, repaint));
        let failure = match spawn {
            Ok(_worker) => None,
            Err(error) => {
                let failure = DocumentFailure::new(
                    DocumentFailureLayer::KatanaHost,
                    "start worker",
                    display_source.uri.clone(),
                    Some(display_source.format),
                    error.to_string(),
                );
                failure.log();
                Some(failure)
            }
        };
        let started = failure.is_none();
        Self {
            generation,
            source: display_source,
            command_tx: Some(command_tx),
            event_rx,
            frame: None,
            border_cache: Default::default(),
            filter_ui: Default::default(),
            fonts: Default::default(),
            failure,
            painter: Default::default(),
            loading: started,
            command_in_flight: started,
            pending_commands: PendingDocumentCommands::default(),
            viewport: None,
            started_at,
        }
    }

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
            ui.centered_and_justified(|ui| {
                ui.spinner();
            });
            return;
        };

        show_controls(self, ui, &frame);
        /* WHY: Place diagnostics before allocating the grid so it cannot overlap the bottom tab input region. */
        show_diagnostics(ui, &frame);
        ui.add_space(4.0);
        let sheet_tab_height = sheet_tab_rail_height(frame.format);
        if sheet_tab_height > 0.0 {
            let rail_frame =
                egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(6, 4));
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
            self.pending_commands.push(command);
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
