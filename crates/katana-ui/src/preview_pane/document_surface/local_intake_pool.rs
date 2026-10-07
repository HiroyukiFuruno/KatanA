use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, Weak};
use std::thread::JoinHandle;

use super::request::IntakeRequest;

pub(super) const WORKER_LIMIT: usize = 2;
pub(super) const QUEUE_LIMIT: usize = 64;

pub(super) struct IntakePool {
    pub(super) shared: Arc<SharedPool>,
    _workers: Vec<JoinHandle<()>>,
}

pub(super) struct SharedPool {
    pub(super) state: Mutex<PoolState>,
    pub(super) wake: Condvar,
}

pub(super) struct PoolState {
    pub(super) phase: PoolPhase,
    pub(super) pending: VecDeque<Weak<IntakeRequest>>,
    pub(super) active: Vec<Weak<IntakeRequest>>,
}

impl PoolState {
    fn capacity_failure_cause(&self, request: &IntakeRequest) -> Option<String> {
        let request_is_active = self
            .active
            .iter()
            .any(|active| std::ptr::eq(active.as_ptr(), request));
        let has_cancelled_read = self
            .active
            .iter()
            .any(|active| active.upgrade().is_some_and(|active| active.is_cancelled()));
        (self.active.len() == WORKER_LIMIT && !request_is_active && has_cancelled_read).then(|| {
            format!("document intake capacity is occupied by cancelled reads (worker limit {WORKER_LIMIT})")
        })
    }
}

pub(super) enum PoolPhase {
    Booting,
    Ready,
    Failed(String),
}

impl IntakePool {
    pub(super) fn global() -> &'static Self {
        static POOL: OnceLock<IntakePool> = OnceLock::new();
        POOL.get_or_init(Self::start)
    }

    fn start() -> Self {
        let shared = Arc::new(SharedPool::new());
        let mut workers = Vec::with_capacity(WORKER_LIMIT);
        for _ in 0..WORKER_LIMIT {
            let worker_shared = shared.clone();
            match std::thread::Builder::new()
                .name("katana-document-intake".to_owned())
                .spawn(move || super::worker::IntakeWorkerOps::run(worker_shared))
            {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    shared.fail(&format!("document intake worker could not start: {error}"));
                    break;
                }
            }
        }
        if workers.len() == WORKER_LIMIT {
            shared.ready();
        }
        Self {
            shared,
            _workers: workers,
        }
    }

    pub(super) fn enqueue(&self, request: &Arc<IntakeRequest>) -> Result<(), String> {
        let mut state = self.shared.lock()?;
        match &state.phase {
            PoolPhase::Ready => {}
            PoolPhase::Failed(cause) => return Err(cause.clone()),
            PoolPhase::Booting => return Err("document intake pool has not started".to_owned()),
        }
        if let Some(cause) = state.capacity_failure_cause(request) {
            return Err(cause);
        }
        state.pending.retain(|request| request.strong_count() > 0);
        if state.pending.len() == QUEUE_LIMIT {
            return Err(format!(
                "document intake queue is full (limit {QUEUE_LIMIT})"
            ));
        }
        state.pending.push_back(Arc::downgrade(request));
        self.shared.wake.notify_one();
        Ok(())
    }

    pub(super) fn failure_cause(&self) -> Option<String> {
        match self.shared.lock() {
            Ok(state) => match &state.phase {
                PoolPhase::Ready => None,
                PoolPhase::Failed(cause) => Some(cause.clone()),
                PoolPhase::Booting => Some("document intake pool has not started".to_owned()),
            },
            Err(cause) => Some(cause),
        }
    }

    pub(super) fn capacity_failure_cause(&self, request: &IntakeRequest) -> Option<String> {
        match self.shared.lock() {
            Ok(state) => state.capacity_failure_cause(request),
            Err(cause) => Some(cause),
        }
    }
}

impl SharedPool {
    fn new() -> Self {
        Self {
            state: Mutex::new(PoolState {
                phase: PoolPhase::Booting,
                pending: VecDeque::new(),
                active: Vec::with_capacity(WORKER_LIMIT),
            }),
            wake: Condvar::new(),
        }
    }

    fn ready(&self) {
        match self.state.lock() {
            Ok(mut state) => {
                if matches!(state.phase, PoolPhase::Booting) {
                    state.phase = PoolPhase::Ready;
                }
                self.wake.notify_all();
            }
            Err(error) => {
                drop(error);
                self.fail("document intake startup lock is poisoned");
            }
        }
    }

    pub(super) fn lock(&self) -> Result<MutexGuard<'_, PoolState>, String> {
        match self.state.lock() {
            Ok(state) => Ok(state),
            Err(error) => {
                drop(error);
                let cause = "document intake queue lock is poisoned";
                self.fail(cause);
                Err(cause.to_owned())
            }
        }
    }

    pub(super) fn fail(&self, cause: &str) {
        let pending = {
            let mut state = match self.state.lock() {
                Ok(state) => state,
                Err(error) => error.into_inner(),
            };
            if !matches!(state.phase, PoolPhase::Failed(_)) {
                state.phase = PoolPhase::Failed(cause.to_owned());
            }
            std::mem::take(&mut state.pending)
        };
        self.wake.notify_all();
        for request in pending.into_iter().filter_map(|request| request.upgrade()) {
            request.fail(cause);
        }
    }

    pub(super) fn release(&self, request: &IntakeRequest) {
        let Ok(mut state) = self.lock() else {
            return;
        };
        state
            .active
            .retain(|active| !std::ptr::eq(active.as_ptr(), request));
    }
}
