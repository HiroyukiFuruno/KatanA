use super::allocator::relieve;
use super::{Arc, Ordering, WorkerMemoryState};

impl WorkerMemoryState {
    pub(super) fn queue_relief_if_idle(self: &Arc<Self>, pending: &mut bool) {
        if !*pending || !self.is_idle() || self.cleanup_running() {
            return;
        }
        *pending = false;
        self.relief_running.store(true, Ordering::Release);
        let epoch = self.relief_epoch.load(Ordering::Acquire);
        let state = Arc::clone(self);
        if let Err(error) = std::thread::Builder::new()
            .name("katana-preview-memory-relief".to_owned())
            .spawn(move || state.run_relief(epoch))
        {
            *pending = true;
            self.relief_running.store(false, Ordering::Release);
            tracing::error!(%error, "document memory relief worker could not start");
        }
    }

    fn is_idle(&self) -> bool {
        self.counter.load(Ordering::Acquire) == 0 && self.intakes.load(Ordering::Acquire) == 0
    }

    fn claim_relief(&self, epoch: u64) -> bool {
        let mut pending = self.pending();
        if epoch != self.relief_epoch.load(Ordering::Acquire) {
            return false;
        }
        if !self.is_idle() {
            *pending = true;
            return false;
        }
        true
    }

    fn next_relief(&self) -> Option<u64> {
        let mut pending = self.pending();
        if *pending && self.is_idle() {
            *pending = false;
            return Some(self.relief_epoch.load(Ordering::Acquire));
        }
        self.relief_running.store(false, Ordering::Release);
        None
    }

    fn run_relief(self: Arc<Self>, mut epoch: u64) {
        loop {
            /* WHY: allocatorの走査中は状態mutexを保持せず、UIの選択・取消を待たせない。 */
            if self.claim_relief(epoch) {
                let released = relieve();
                super::super::debug_log::DebugLog::write(
                    "document_memory_relief",
                    format_args!("released_bytes={released}"),
                );
            }
            let Some(next) = self.next_relief() else {
                break;
            };
            epoch = next;
        }
    }
}

#[cfg(test)]
#[path = "reclaim_tests.rs"]
mod tests;
