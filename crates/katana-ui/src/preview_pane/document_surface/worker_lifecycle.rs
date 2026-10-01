use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE_DOCUMENT_WORKERS: AtomicUsize = AtomicUsize::new(0);

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
        LIVE_DOCUMENT_WORKERS.load(Ordering::Acquire)
    }
}

struct DocumentWorkerLease<'a> {
    counter: &'a AtomicUsize,
}

impl DocumentWorkerLease<'static> {
    fn start() -> Self {
        Self::acquire(&LIVE_DOCUMENT_WORKERS)
    }
}

impl<'a> DocumentWorkerLease<'a> {
    fn acquire(counter: &'a AtomicUsize) -> Self {
        counter.fetch_add(1, Ordering::AcqRel);
        Self { counter }
    }
}

impl Drop for DocumentWorkerLease<'_> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
#[path = "worker_lifecycle_tests.rs"]
mod tests;
