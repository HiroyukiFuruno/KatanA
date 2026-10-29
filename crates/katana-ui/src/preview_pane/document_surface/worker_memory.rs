use std::sync::{
    Arc, LazyLock, Mutex, MutexGuard,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

mod allocator;
mod reclaim;

pub(super) static DOCUMENT_WORKER_MEMORY: LazyLock<Arc<WorkerMemoryState>> =
    LazyLock::new(|| Arc::new(WorkerMemoryState::new()));

#[derive(Debug)]
pub(super) struct WorkerMemoryState {
    pub(super) counter: AtomicUsize,
    intakes: AtomicUsize,
    relief_pending: Mutex<bool>,
    relief_running: AtomicBool,
    relief_epoch: AtomicU64,
}

#[derive(Debug)]
pub(super) struct IntakeMemoryLease {
    state: Arc<WorkerMemoryState>,
}

impl Drop for IntakeMemoryLease {
    fn drop(&mut self) {
        let mut pending = self.state.pending();
        self.state.intakes.fetch_sub(1, Ordering::AcqRel);
        self.state.queue_relief_if_idle(&mut pending);
    }
}

impl WorkerMemoryState {
    pub(super) const fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            intakes: AtomicUsize::new(0),
            relief_pending: Mutex::new(false),
            relief_running: AtomicBool::new(false),
            relief_epoch: AtomicU64::new(0),
        }
    }

    fn pending(&self) -> MutexGuard<'_, bool> {
        self.relief_pending.lock().unwrap_or_else(|error| {
            tracing::error!("document worker memory state was poisoned");
            error.into_inner()
        })
    }

    pub(super) fn acquire(&self) {
        let mut pending = self.pending();
        /* WHY: 新しい文書が始まった場合、以前の最終タブ終了要求を持ち越さない。 */
        *pending = false;
        self.relief_epoch.fetch_add(1, Ordering::AcqRel);
        self.counter.fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn retain_intake(self: &Arc<Self>) -> IntakeMemoryLease {
        let _pending = self.pending();
        self.intakes.fetch_add(1, Ordering::AcqRel);
        IntakeMemoryLease {
            state: Arc::clone(self),
        }
    }

    pub(super) fn release(self: &Arc<Self>) {
        let mut pending = self.pending();
        self.counter.fetch_sub(1, Ordering::AcqRel);
        self.queue_relief_if_idle(&mut pending);
    }

    pub(super) fn request_relief(self: &Arc<Self>) {
        let mut pending = self.pending();
        *pending = true;
        self.queue_relief_if_idle(&mut pending);
    }

    pub(super) fn cancel_relief(&self) {
        /* WHY: HTMLや画像はworker leaseを持たないため、再表示をhost側から通知する。 */
        let mut pending = self.pending();
        *pending = false;
        self.relief_epoch.fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn cleanup_running(&self) -> bool {
        self.relief_running.load(Ordering::Acquire)
    }

    pub(super) fn activity_count(&self) -> usize {
        /* WHY: worker終了から返却開始への引継ぎ途中をidleと観測させない。 */
        let _pending = self.pending();
        self.counter.load(Ordering::Acquire) + usize::from(self.cleanup_running())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relief_remains_pending_until_last_worker_releases() {
        let state = Arc::new(WorkerMemoryState::new());
        state.acquire();
        state.acquire();
        state.request_relief();
        assert!(*state.pending(), "live workers must defer memory relief");
        state.release();
        assert!(*state.pending(), "the second worker still owns memory");
        state.release();
        assert!(!*state.pending());
        assert_eq!(state.counter.load(Ordering::Acquire), 0);
    }

    #[test]
    fn a_new_worker_cancels_previous_close_request() {
        let state = Arc::new(WorkerMemoryState::new());
        state.acquire();
        state.request_relief();
        state.acquire();
        assert!(!*state.pending());
        state.release();
        state.release();
        assert!(!*state.pending());
    }

    #[test]
    fn reopening_a_non_worker_document_cancels_pending_relief() {
        let state = Arc::new(WorkerMemoryState::new());
        state.acquire();
        state.request_relief();
        assert!(*state.pending());
        state.cancel_relief();
        assert!(
            !*state.pending(),
            "reopened content must cancel deferred relief"
        );
        state.release();
        assert!(!*state.pending());
    }

    #[test]
    fn idle_request_is_consumed_without_waiting_for_a_future_worker() {
        let state = Arc::new(WorkerMemoryState::new());
        state.request_relief();
        assert!(!*state.pending());
        assert_eq!(state.counter.load(Ordering::Acquire), 0);
    }

    #[test]
    fn intake_payload_defers_relief_without_changing_worker_count() {
        let state = Arc::new(WorkerMemoryState::new());
        let intake = state.retain_intake();
        state.request_relief();
        assert!(*state.pending());
        assert_eq!(state.counter.load(Ordering::Acquire), 0);
        let queued_intake = state.retain_intake();
        assert!(*state.pending(), "intake preserves pending relief");
        drop(intake);
        assert!(*state.pending());
        drop(queued_intake);
        assert!(!*state.pending());
    }

    #[test]
    fn both_worker_and_intake_must_release_before_relief() {
        for intake_first in [true, false] {
            let state = Arc::new(WorkerMemoryState::new());
            let intake = state.retain_intake();
            state.acquire();
            state.request_relief();
            if intake_first {
                drop(intake);
                assert!(*state.pending());
                state.release();
            } else {
                state.release();
                assert!(*state.pending());
                drop(intake);
            }
            assert!(!*state.pending());
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_pressure_relief_preserves_live_allocations() {
        let live = vec![0x2a_u8; 32];
        let pointer = live.as_ptr();
        let contents = live.clone();
        let _released = allocator::relieve();
        assert_eq!(live.as_ptr(), pointer);
        assert_eq!(live, contents);
    }
}
