use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use super::CanvasRevision;
use super::input::CanvasInput;
use eframe::egui;
use katana_core::canvas::{CanvasDocument, CanvasError};

#[cfg(test)]
pub(super) mod refresh_test_sync;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanvasRequestError {
    WorkerStart(std::io::ErrorKind),
    QueueFull,
    PoolPoisoned,
    RequestPoisoned,
    WorkerStopped,
    Cancelled,
    ResultDisconnected,
    SourceUnavailable,
    InvalidCanvas(CanvasError),
    FileRead(std::io::ErrorKind),
    NotRegularFile,
    SizeLimit,
    InvalidUtf8,
}

#[derive(Debug)]
pub(crate) struct CanvasParseResult {
    pub generation: u64,
    pub revision: Option<CanvasRevision>,
    pub result: Result<Arc<CanvasDocument>, CanvasRequestError>,
}

pub(super) struct RequestState {
    pub generation: u64,
    pub source: Mutex<Option<CanvasInput>>,
    cancelled: AtomicBool,
    completed: AtomicBool,
    sender: Sender<CanvasParseResult>,
    repaint: Option<egui::Context>,
}

impl RequestState {
    pub(super) fn new(
        generation: u64,
        source: impl Into<CanvasInput>,
        repaint: Option<egui::Context>,
    ) -> (Arc<Self>, Receiver<CanvasParseResult>) {
        let (sender, receiver) = std::sync::mpsc::channel();
        let request = Self {
            generation,
            source: Mutex::new(Some(source.into())),
            cancelled: AtomicBool::new(false),
            completed: AtomicBool::new(false),
            sender,
            repaint,
        };
        (Arc::new(request), receiver)
    }

    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(super) fn take_input(&self) -> Result<CanvasInput, CanvasRequestError> {
        let input = self
            .source
            .lock()
            .map_err(|_| CanvasRequestError::RequestPoisoned)?
            .take()
            .ok_or(CanvasRequestError::SourceUnavailable)?;
        Ok(input)
    }

    pub(super) fn deliver(
        &self,
        result: Result<Arc<CanvasDocument>, CanvasRequestError>,
    ) -> Result<(), CanvasRequestError> {
        self.deliver_revision(result, None)
    }

    pub(super) fn deliver_revision(
        &self,
        result: Result<Arc<CanvasDocument>, CanvasRequestError>,
        revision: Option<CanvasRevision>,
    ) -> Result<(), CanvasRequestError> {
        if self.is_cancelled() || self.completed.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        self.sender
            .send(CanvasParseResult {
                generation: self.generation,
                revision,
                result,
            })
            .map_err(|_| CanvasRequestError::ResultDisconnected)?;
        if let Some(repaint) = &self.repaint {
            repaint.request_repaint();
        }
        Ok(())
    }
}
