use eframe::egui;
use katana_document_viewer::{
    DocumentFitMode, DocumentSurfaceCommand, DocumentSurfaceHost, DocumentViewerCommand,
};
use std::sync::atomic::{AtomicU64, Ordering};

use super::controls::show_controls;
use super::render_support::{PendingDocumentCommands, show_diagnostics, show_failure};
use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker::{self, DocumentWorkerCommand};

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

impl DocumentSurface {
    pub(crate) fn start(source: DocumentSurfaceSource, ctx: &egui::Context) -> Self {
        let generation = NEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
        let (command_tx, command_rx) = std::sync::mpsc::sync_channel(1);
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let display_source = source.descriptor();
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
            failure,
            host: DocumentSurfaceHost::default(),
            loading: started,
            command_in_flight: started,
            pending_commands: PendingDocumentCommands::default(),
            viewport: None,
        }
    }

    pub(crate) fn source(&self) -> &DocumentSurfaceSource {
        &self.source
    }

    pub(crate) fn frame_state_for_test(&self) -> Option<(String, usize, usize, String)> {
        let frame = self.frame.as_ref()?;
        Some((
            frame.format.extension().to_owned(),
            frame.state.active_index,
            frame.state.item_count,
            format!("{:?}", frame.surface.kind()),
        ))
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
        ui.add_space(4.0);
        let output = self.host.show(ui, &frame.surface, self.generation);
        for command in output.into_commands() {
            self.queue_surface(command);
        }
        show_diagnostics(ui, &frame);
        self.frame = Some(frame);
    }

    pub(super) fn queue(&mut self, command: DocumentWorkerCommand) {
        if self.command_in_flight {
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
