use std::sync::Arc;
use std::sync::atomic::Ordering;

use katana_core::canvas::CanvasDocument;
use sha2::{Digest, Sha256};

use super::state::{CanvasRequestError, RequestState};
use super::{CanvasRevision, pool::SharedPool};

pub(super) struct CanvasWorker;

const REVISION_CHUNK_BYTES: usize = 64 * 1024;

struct ParsedCanvas {
    revision: CanvasRevision,
    result: Result<Arc<CanvasDocument>, CanvasRequestError>,
}

struct WorkerExit(Arc<SharedPool>);

impl Drop for WorkerExit {
    fn drop(&mut self) {
        self.0.finish();
    }
}

impl CanvasWorker {
    pub(super) fn run(shared: Arc<SharedPool>) {
        let _exit = WorkerExit(Arc::clone(&shared));
        loop {
            match Self::next_request(&shared) {
                Ok(Some(request)) => {
                    if let Err(error) = Self::process(&request) {
                        tracing::error!(
                            ?error,
                            generation = request.generation,
                            "Canvas result delivery failed"
                        );
                    }
                    if let Err(error) = shared.release() {
                        tracing::error!(?error, "Canvas active-request release failed");
                        return;
                    }
                }
                Ok(None) => return,
                Err(error) => {
                    tracing::error!(?error, "Canvas parser queue failed");
                    return;
                }
            }
        }
    }

    fn next_request(shared: &SharedPool) -> Result<Option<Arc<RequestState>>, CanvasRequestError> {
        let mut state = shared
            .state
            .lock()
            .map_err(|_| CanvasRequestError::PoolPoisoned)?;
        loop {
            if shared.stopping.load(Ordering::Acquire) {
                return Ok(None);
            }
            if let Some(request) = state.pending.pop_front() {
                if let Some(request) = request.upgrade().filter(|request| !request.is_cancelled()) {
                    state.active = Arc::downgrade(&request);
                    shared.wake.notify_all();
                    return Ok(Some(request));
                }
                continue;
            }
            state = shared
                .wake
                .wait(state)
                .map_err(|_| CanvasRequestError::PoolPoisoned)?;
        }
    }

    pub(super) fn process(request: &RequestState) -> Result<(), CanvasRequestError> {
        if request.is_cancelled() {
            return Ok(());
        }
        let source = match request.take_source() {
            Ok(source) => source,
            Err(error) => return request.deliver(Err(error)),
        };
        let Some(parsed) = Self::parse_source(&source, || request.is_cancelled()) else {
            return Ok(());
        };
        request.deliver_revision(parsed.result, Some(parsed.revision))
    }

    fn parse_source(source: &str, cancelled: impl Fn() -> bool) -> Option<ParsedCanvas> {
        let revision = Self::revision(source, &cancelled)?;
        let result = match CanvasDocument::parse_cancellable(source, cancelled) {
            Ok(Some(document)) => Ok(Arc::new(document)),
            Ok(None) => return None,
            Err(error) => Err(CanvasRequestError::InvalidCanvas(error)),
        };
        Some(ParsedCanvas { revision, result })
    }

    fn revision(source: &str, cancelled: impl Fn() -> bool) -> Option<CanvasRevision> {
        let mut digest = Sha256::new();
        for chunk in source.as_bytes().chunks(REVISION_CHUNK_BYTES) {
            if cancelled() {
                return None;
            }
            digest.update(chunk);
        }
        if cancelled() {
            return None;
        }
        Some(digest.finalize().into())
    }
}

#[cfg(test)]
mod cancellation_tests;
