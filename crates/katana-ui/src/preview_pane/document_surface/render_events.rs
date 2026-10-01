use eframe::egui;
use std::sync::mpsc::{TryRecvError, TrySendError};

use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker::{DocumentWorkerCommand, DocumentWorkerEvent};

const DOCUMENT_WORKER_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

impl DocumentSurface {
    pub(super) fn poll(&mut self, ctx: &egui::Context) {
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => self.apply_event(ctx, event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if self.loading || self.command_in_flight {
                        self.fail_disconnected();
                    }
                    break;
                }
            }
        }
        self.fonts.poll(self.generation);
        if !self.command_in_flight
            && let Some(command) = self.pending_commands.take_next()
        {
            self.send(command);
        }
        if self.loading {
            ctx.request_repaint_after(DOCUMENT_WORKER_POLL_INTERVAL);
        }
    }

    pub(super) fn apply_event(&mut self, ctx: &egui::Context, event: DocumentWorkerEvent) {
        let event_generation = match &event {
            DocumentWorkerEvent::Frame { generation, .. }
            | DocumentWorkerEvent::Failure { generation, .. } => *generation,
        };
        if event_generation != self.generation {
            return;
        }
        self.loading = false;
        self.command_in_flight = false;
        match event {
            DocumentWorkerEvent::Frame {
                frame,
                border_cache,
                font_requests,
                session_event,
                spreadsheet_metadata,
                filter_event,
                ..
            } => {
                super::debug_log::DebugLog::write(
                    "document_frame_received",
                    format_args!(
                        "generation={} format={} active_index={} item_count={} surface={:?} elapsed_ms={}",
                        self.generation,
                        super::worker_support::format_extension(frame.format),
                        frame.state.active_index,
                        frame.state.item_count,
                        frame.surface.kind(),
                        self.started_at.elapsed().as_millis()
                    ),
                );
                tracing::debug!(
                    generation = self.generation,
                    format = super::worker_support::format_extension(frame.format),
                    active_index = frame.state.active_index,
                    item_count = frame.state.item_count,
                    surface = ?frame.surface.kind(),
                    "received document frame"
                );
                apply_platform_event(ctx, &frame, session_event);
                self.filter_ui
                    .update_metadata(spreadsheet_metadata, filter_event.as_ref());
                self.border_cache = border_cache;
                self.frame = Some(*frame);
                self.failure = None;
                self.fonts.update(self.generation, font_requests, ctx);
            }
            DocumentWorkerEvent::Failure { failure, .. } => {
                failure.log();
                self.failure = Some(failure);
                self.command_tx.take();
                self.pending_commands.clear();
                self.fonts.cancel();
            }
        }
    }

    pub(super) fn send(&mut self, command: DocumentWorkerCommand) {
        let Some(sender) = &self.command_tx else {
            self.fail_disconnected();
            return;
        };
        tracing::debug!(
            generation = self.generation,
            ?command,
            "sending document command"
        );
        match sender.try_send(command) {
            Ok(()) => {
                self.command_in_flight = true;
                self.loading = self.frame.is_none();
            }
            Err(TrySendError::Full(command)) => {
                tracing::debug!(
                    generation = self.generation,
                    ?command,
                    "document worker channel was full; preserving command"
                );
                self.pending_commands.push(command);
            }
            Err(TrySendError::Disconnected(_)) => self.fail_disconnected(),
        }
    }

    fn fail_disconnected(&mut self) {
        self.loading = false;
        self.command_in_flight = false;
        let failure = DocumentFailure::new(
            DocumentFailureLayer::KdvWorker,
            "communicate",
            self.source.uri.clone(),
            Some(self.source.format),
            "document worker stopped before completing the requested operation",
        );
        failure.log();
        self.failure = Some(failure);
        self.command_tx.take();
        self.pending_commands.clear();
        self.fonts.cancel();
    }
}

fn apply_platform_event(
    ctx: &egui::Context,
    frame: &katana_document_viewer::DocumentFrame,
    event: katana_document_viewer::DocumentSessionEvent,
) {
    if event
        == katana_document_viewer::DocumentSessionEvent::Viewer(
            katana_document_viewer::DocumentViewerEvent::CopyRequested,
        )
        && let Some(text) = frame.surface.active_text()
    {
        ctx.copy_text(text.to_owned());
    }
}
