use super::worker_memory::{DOCUMENT_WORKER_MEMORY, WorkerMemoryState};

pub(crate) struct DocumentWorkerLifecycle;

impl DocumentWorkerLifecycle {
    pub(crate) fn spawn(
        name: String,
        work: impl FnOnce() + Send + 'static,
    ) -> std::io::Result<std::thread::JoinHandle<()>> {
        let worker_inputs = (work, DocumentWorkerLease::start());
        std::thread::Builder::new().name(name).spawn(move || {
            let (work, _worker_lifetime) = std::convert::identity(worker_inputs);
            work();
        })
    }

    pub(crate) fn live_count() -> usize {
        /* WHY: 返却処理が残る間は文書lifecycleをidleと記録しない。 */
        DOCUMENT_WORKER_MEMORY.activity_count()
    }

    pub(crate) fn request_closed_preview_memory_relief() {
        DOCUMENT_WORKER_MEMORY.request_relief();
    }

    pub(crate) fn cancel_closed_preview_memory_relief() {
        DOCUMENT_WORKER_MEMORY.cancel_relief();
    }
}

struct DocumentWorkerLease {
    state: std::sync::Arc<WorkerMemoryState>,
}

impl DocumentWorkerLease {
    fn start() -> Self {
        Self::acquire(&DOCUMENT_WORKER_MEMORY)
    }
}

impl DocumentWorkerLease {
    fn acquire(state: &std::sync::Arc<WorkerMemoryState>) -> Self {
        state.acquire();
        Self {
            state: std::sync::Arc::clone(state),
        }
    }
}

impl Drop for DocumentWorkerLease {
    fn drop(&mut self) {
        self.state.release();
    }
}

#[cfg(test)]
#[path = "worker_lifecycle_tests.rs"]
mod tests;
