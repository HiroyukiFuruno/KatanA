use super::worker_memory::WorkerMemoryState;
use std::sync::atomic::Ordering;

static DOCUMENT_WORKER_MEMORY: WorkerMemoryState = WorkerMemoryState::new();

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
        DOCUMENT_WORKER_MEMORY.counter.load(Ordering::Acquire)
    }

    pub(crate) fn request_closed_preview_memory_relief() {
        DOCUMENT_WORKER_MEMORY.request_relief();
    }
}

struct DocumentWorkerLease<'a> {
    state: &'a WorkerMemoryState,
}

impl DocumentWorkerLease<'static> {
    fn start() -> Self {
        Self::acquire(&DOCUMENT_WORKER_MEMORY)
    }
}

impl<'a> DocumentWorkerLease<'a> {
    fn acquire(state: &'a WorkerMemoryState) -> Self {
        state.acquire();
        Self { state }
    }
}

impl Drop for DocumentWorkerLease<'_> {
    fn drop(&mut self) {
        self.state.release();
    }
}

#[cfg(test)]
#[path = "worker_lifecycle_tests.rs"]
mod tests;
