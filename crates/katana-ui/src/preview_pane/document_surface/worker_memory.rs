use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicUsize, Ordering},
};

pub(super) struct WorkerMemoryState {
    pub(super) counter: AtomicUsize,
    relief_pending: Mutex<bool>,
}

impl WorkerMemoryState {
    pub(super) const fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            relief_pending: Mutex::new(false),
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
        self.counter.fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn release(&self) {
        let mut pending = self.pending();
        self.counter.fetch_sub(1, Ordering::AcqRel);
        self.relieve_if_idle(&mut pending);
    }

    pub(super) fn request_relief(&self) {
        let mut pending = self.pending();
        *pending = true;
        self.relieve_if_idle(&mut pending);
    }

    pub(super) fn cancel_relief(&self) {
        /* WHY: HTMLや画像はworker leaseを持たないため、再表示をhost側から通知する。 */
        *self.pending() = false;
    }

    fn relieve_if_idle(&self, pending: &mut bool) {
        /* WHY: 所有データの解放後にだけ返却し、返却中の新worker開始も同じmutexで順序化する。 */
        if *pending && self.counter.load(Ordering::Acquire) == 0 {
            *pending = false;
            let _released = relieve();
        }
    }
}

#[cfg(target_os = "macos")]
fn relieve() -> usize {
    use std::ffi::c_void;
    unsafe extern "C" {
        fn malloc_zone_pressure_relief(zone: *mut c_void, goal: usize) -> usize;
    }
    /* SAFETY: NULLで登録済みzone全体を対象にし、goal 0で最大限の返却候補を要求する。
     * ポインタを保持せず、呼出し中だけlibmallocの契約に従う。 */
    unsafe { malloc_zone_pressure_relief(std::ptr::null_mut(), 0) }
}

#[cfg(not(target_os = "macos"))]
fn relieve() -> usize {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relief_remains_pending_until_last_worker_releases() {
        let state = WorkerMemoryState::new();
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
        let state = WorkerMemoryState::new();
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
        let state = WorkerMemoryState::new();
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
        let state = WorkerMemoryState::new();
        state.request_relief();
        assert!(!*state.pending());
        assert_eq!(state.counter.load(Ordering::Acquire), 0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_pressure_relief_preserves_live_allocations() {
        let live = vec![0x2a_u8; 32];
        let pointer = live.as_ptr();
        let contents = live.clone();
        let _released = relieve();
        assert_eq!(live.as_ptr(), pointer);
        assert_eq!(live, contents);
    }
}
