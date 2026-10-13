use std::sync::Arc;
use std::sync::atomic::Ordering;

use katana_core::canvas::CanvasDocument;
use sha2::{Digest, Sha256};

use super::pool::SharedPool;
use super::state::{CanvasRequestError, RequestState};

pub(super) struct CanvasWorker;

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
        if request.is_cancelled() {
            return Ok(());
        }
        let revision = Sha256::digest(source.as_bytes()).into();
        if request.is_cancelled() {
            return Ok(());
        }
        let result = CanvasDocument::parse(&source)
            .map(Arc::new)
            .map_err(CanvasRequestError::InvalidCanvas);
        /* WHY: 同期 parser を中断せず、取消後の結果を次世代の表示へ届けない。 */
        request.deliver_revision(result, Some(revision))
    }
}
