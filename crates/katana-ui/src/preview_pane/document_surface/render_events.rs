use eframe::egui;
use std::sync::mpsc::{TryRecvError, TrySendError};

use super::types::{DocumentFailure, DocumentFailureLayer, DocumentSurface};
use super::worker::{DocumentWorkerCommand, DocumentWorkerEvent};

const DOCUMENT_WORKER_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

impl DocumentSurface {
    pub(super) fn poll(&mut self, ctx: &egui::Context) {
        loop {
            match self.event_rx.try_recv() {
                Ok(event) => self.apply_event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if self.loading || self.command_in_flight {
                        self.fail_disconnected();
                    }
                    break;
                }
            }
        }
        if !self.command_in_flight
            && let Some(command) = self.pending_commands.take_next()
        {
            self.send(command);
        }
        if self.loading {
            ctx.request_repaint_after(DOCUMENT_WORKER_POLL_INTERVAL);
        }
    }

    fn apply_event(&mut self, event: DocumentWorkerEvent) {
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
            DocumentWorkerEvent::Frame { frame, .. } => {
                self.frame = Some(*frame);
                self.failure = None;
            }
            DocumentWorkerEvent::Failure { failure, .. } => {
                failure.log();
                self.failure = Some(failure);
                self.command_tx.take();
                self.pending_commands.clear();
            }
        }
    }

    pub(super) fn send(&mut self, command: DocumentWorkerCommand) {
        let Some(sender) = &self.command_tx else {
            self.fail_disconnected();
            return;
        };
        match sender.try_send(command) {
            Ok(()) => {
                self.command_in_flight = true;
                self.loading = self.frame.is_none();
            }
            Err(TrySendError::Full(command)) => {
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
    }
}
