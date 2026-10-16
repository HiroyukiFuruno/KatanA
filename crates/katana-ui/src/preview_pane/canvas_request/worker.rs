use std::sync::Arc;
use std::sync::atomic::Ordering;

use katana_core::canvas::CanvasDocument;
use sha2::{Digest, Sha256};

use super::input::CanvasInput;
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
        let input = match request.take_input() {
            Ok(input) => input,
            Err(error) => return request.deliver(Err(error)),
        };
        let parsed = match Self::parse_input(input, || request.is_cancelled()) {
            Ok(Some(parsed)) => parsed,
            Ok(None) => return Ok(()),
            Err(error) => return request.deliver(Err(error)),
        };
        request.deliver_revision(parsed.result, Some(parsed.revision))
    }

    fn parse_input(
        input: CanvasInput,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<ParsedCanvas>, CanvasRequestError> {
        match input {
            CanvasInput::Snapshot {
                path,
                revision,
                document,
            } => Self::parse_snapshot(path, revision, document, cancelled),
            input => Ok(Self::parse_source(&input.load(&cancelled)?, &cancelled)),
        }
    }

    fn parse_snapshot(
        path: std::path::PathBuf,
        expected: CanvasRevision,
        document: Arc<CanvasDocument>,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<ParsedCanvas>, CanvasRequestError> {
        let source = CanvasInput::File(path).load(&cancelled)?;
        let revision = Self::revision(&source, &cancelled);
        let Some(revision) = revision else {
            return Ok(None);
        };
        if revision == expected {
            return Ok(Some(ParsedCanvas {
                revision,
                result: Ok(document),
            }));
        }
        Ok(Self::parse_document(&source, revision, cancelled))
    }

    fn parse_source(source: &str, cancelled: impl Fn() -> bool) -> Option<ParsedCanvas> {
        let revision = Self::revision(source, &cancelled)?;
        Self::parse_document(source, revision, cancelled)
    }

    fn parse_document(
        source: &str,
        revision: CanvasRevision,
        cancelled: impl Fn() -> bool,
    ) -> Option<ParsedCanvas> {
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
