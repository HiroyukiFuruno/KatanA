use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};

use eframe::egui;

mod input;
mod pool;
mod queue;
mod state;
mod worker;

#[cfg(test)]
mod active_cancellation_tests;

pub(crate) use state::{CanvasParseResult, CanvasRequestError};

#[cfg(test)]
pub(in crate::preview_pane) use state::refresh_test_sync::wait_until_completed;

use input::CanvasInput;
use pool::{CanvasPool, SharedPool};
use state::RequestState;

const SHA256_DIGEST_BYTES: usize = 32;
pub(crate) type CanvasRevision = [u8; SHA256_DIGEST_BYTES];

pub(crate) struct CanvasParseRequest {
    request: Arc<RequestState>,
    receiver: Receiver<CanvasParseResult>,
    shared: Arc<SharedPool>,
    refresh_requested: bool,
}

impl CanvasParseRequest {
    pub(crate) fn start(
        generation: u64,
        source: String,
        repaint: Option<egui::Context>,
    ) -> Result<Self, CanvasRequestError> {
        Self::submit(CanvasPool::global()?, generation, source, repaint)
    }

    fn submit(
        pool: &CanvasPool,
        generation: u64,
        source: String,
        repaint: Option<egui::Context>,
    ) -> Result<Self, CanvasRequestError> {
        Self::submit_input(pool, generation, CanvasInput::Text(source), repaint)
    }

    pub(crate) fn start_path(
        generation: u64,
        path: PathBuf,
        repaint: Option<egui::Context>,
    ) -> Result<Self, CanvasRequestError> {
        Self::submit_input(
            CanvasPool::global()?,
            generation,
            CanvasInput::File(path),
            repaint,
        )
    }

    fn submit_input(
        pool: &CanvasPool,
        generation: u64,
        input: CanvasInput,
        repaint: Option<egui::Context>,
    ) -> Result<Self, CanvasRequestError> {
        let (request, receiver) = RequestState::new(generation, input, repaint);
        pool.shared.enqueue(&request)?;
        Ok(Self {
            request,
            receiver,
            shared: Arc::clone(&pool.shared),
            refresh_requested: false,
        })
    }

    pub(crate) fn request_refresh(&mut self) {
        self.refresh_requested = true;
    }

    pub(crate) fn refresh_requested(&self) -> bool {
        self.refresh_requested
    }

    pub(crate) fn poll(&self) -> Result<Option<CanvasParseResult>, CanvasRequestError> {
        if self.request.is_cancelled() {
            return Err(CanvasRequestError::Cancelled);
        }
        match self.receiver.try_recv() {
            Ok(result) => Ok(Some(result)),
            Err(TryRecvError::Empty) => {
                self.shared.check_running()?;
                Ok(None)
            }
            Err(TryRecvError::Disconnected) => Err(CanvasRequestError::ResultDisconnected),
        }
    }

    pub(crate) fn cancel(&self) {
        self.request.cancel();
    }
}

impl Drop for CanvasParseRequest {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod queue_tests;
#[cfg(test)]
mod revision_tests;
#[cfg(test)]
mod tests;
