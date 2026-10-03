use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use super::{DocumentFailure, DocumentSurfaceSource};

pub(super) type IntakeResult = Result<DocumentSurfaceSource, DocumentFailure>;

#[derive(Debug)]
pub(super) struct IntakeRequest {
    pub(super) path: PathBuf,
    cancelled: AtomicBool,
    sender: Mutex<Option<Sender<IntakeResult>>>,
}

impl IntakeRequest {
    pub(super) fn new(path: PathBuf) -> (Arc<Self>, Receiver<IntakeResult>) {
        let (sender, receiver) = std::sync::mpsc::channel();
        let request = Self {
            path,
            cancelled: AtomicBool::new(false),
            sender: Mutex::new(Some(sender)),
        };
        (Arc::new(request), receiver)
    }

    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(super) fn take_sender(&self) -> Result<Option<Sender<IntakeResult>>, String> {
        self.sender
            .lock()
            .map(|mut sender| sender.take())
            .map_err(|_| "document intake result lock is poisoned".to_owned())
    }

    #[cfg(test)]
    pub(super) fn sender_is_taken(&self) -> bool {
        self.sender.lock().expect("result sender lock").is_none()
    }

    pub(super) fn failure(&self, cause: impl Into<String>) -> DocumentFailure {
        DocumentFailure::new(
            super::super::types::DocumentFailureLayer::KatanaHost,
            "start intake",
            self.path.display().to_string(),
            katana_core::document_source::BinaryDocumentFormat::from_path(&self.path),
            cause,
        )
    }

    pub(super) fn fail(&self, cause: &str) {
        let mut sender = match self.sender.lock() {
            Ok(sender) => sender,
            Err(error) => error.into_inner(),
        };
        if let Some(sender) = sender.take() {
            let _ = sender.send(Err(self.failure(cause)));
        }
    }
}
