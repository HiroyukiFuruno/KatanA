use std::sync::Arc;

use super::DocumentSurfaceSource;
use super::pool::{PoolPhase, SharedPool};
use super::request::IntakeRequest;
use super::request::IntakeResult;

#[cfg(test)]
#[path = "local_intake_worker_tests.rs"]
mod tests;

struct WorkerExitGuard(Arc<SharedPool>);

impl Drop for WorkerExitGuard {
    fn drop(&mut self) {
        self.0.fail("document intake worker stopped");
    }
}

pub(super) struct IntakeWorkerOps;

impl IntakeWorkerOps {
    pub(super) fn run(shared: Arc<SharedPool>) {
        let _exit_guard = WorkerExitGuard(shared.clone());
        loop {
            match Self::next_request(&shared) {
                Ok(Some(request)) => Self::process_request(&shared, &request),
                Ok(None) => return,
                Err(cause) => {
                    shared.fail(&cause);
                    return;
                }
            }
        }
    }
    fn next_request(shared: &SharedPool) -> Result<Option<Arc<IntakeRequest>>, String> {
        let mut state = shared.lock()?;
        loop {
            match &state.phase {
                PoolPhase::Failed(_) => return Ok(None),
                PoolPhase::Ready => {
                    if let Some(request) = state.pending.pop_front() {
                        if let Some(request) = request.upgrade() {
                            return Ok(Some(request));
                        }
                        continue;
                    }
                }
                PoolPhase::Booting => {}
            }
            state = shared
                .wake
                .wait(state)
                .map_err(|_| "document intake queue wait lock is poisoned".to_owned())?;
        }
    }

    fn process_request(shared: &SharedPool, request: &IntakeRequest) {
        if request.is_cancelled() {
            return;
        }
        let sender = match request.take_sender() {
            Ok(Some(sender)) => sender,
            Ok(None) => return,
            Err(cause) => {
                request.fail(&cause);
                shared.fail(&cause);
                return;
            }
        };
        let result = DocumentSurfaceSource::local(&request.path);
        Self::deliver(shared, request, sender, result);
    }

    fn deliver(
        shared: &SharedPool,
        request: &IntakeRequest,
        sender: std::sync::mpsc::Sender<IntakeResult>,
        result: IntakeResult,
    ) {
        if let Ok(state) = shared.lock()
            && matches!(state.phase, PoolPhase::Ready)
            && !request.is_cancelled()
        {
            let _ = sender.send(result);
        }
    }
}
