use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, Weak};
use std::thread::JoinHandle;

use super::queue::QueueState;
use super::state::{CanvasRequestError, RequestState};
use super::worker::CanvasWorker;

pub(super) const QUEUE_LIMIT: usize = 64;

pub(super) struct CanvasPool {
    pub shared: Arc<SharedPool>,
    worker: Option<JoinHandle<()>>,
}

pub(super) struct SharedPool {
    pub state: Mutex<QueueState>,
    pub wake: Condvar,
    pub running: AtomicBool,
    pub stopping: AtomicBool,
}

impl CanvasPool {
    pub(super) fn global() -> Result<&'static Self, CanvasRequestError> {
        static POOL: OnceLock<Result<CanvasPool, CanvasRequestError>> = OnceLock::new();
        POOL.get_or_init(Self::start).as_ref().map_err(Clone::clone)
    }

    pub(super) fn start() -> Result<Self, CanvasRequestError> {
        let shared = Arc::new(SharedPool::new());
        let worker_shared = Arc::clone(&shared);
        let worker = std::thread::Builder::new()
            .name("katana-canvas-parser".to_owned())
            .spawn(move || CanvasWorker::run(worker_shared))
            .map_err(|error| CanvasRequestError::WorkerStart(error.kind()))?;
        Ok(Self {
            shared,
            worker: Some(worker),
        })
    }
}

impl Drop for CanvasPool {
    fn drop(&mut self) {
        self.shared.shutdown();
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            tracing::error!("Canvas parser worker stopped with a panic");
        }
    }
}

impl SharedPool {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(QueueState::default()),
            wake: Condvar::new(),
            running: AtomicBool::new(true),
            stopping: AtomicBool::new(false),
        }
    }

    pub(super) fn check_running(&self) -> Result<(), CanvasRequestError> {
        drop(
            self.state
                .lock()
                .map_err(|_| CanvasRequestError::PoolPoisoned)?,
        );
        if !self.running.load(Ordering::Acquire) || self.stopping.load(Ordering::Acquire) {
            return Err(CanvasRequestError::WorkerStopped);
        }
        Ok(())
    }

    pub(super) fn enqueue(&self, request: &Arc<RequestState>) -> Result<(), CanvasRequestError> {
        self.check_running()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| CanvasRequestError::PoolPoisoned)?;
        if !self.running.load(Ordering::Acquire) || self.stopping.load(Ordering::Acquire) {
            return Err(CanvasRequestError::WorkerStopped);
        }
        /* WHY: 待ち行列は Weak のみを保持し、閉じたタブの本文を保全し続けない。 */
        state.pending.retain(|request| {
            request
                .upgrade()
                .is_some_and(|request| !request.is_cancelled())
        });
        if state.pending.len() >= QUEUE_LIMIT {
            return Err(CanvasRequestError::QueueFull);
        }
        state.pending.push_back(Arc::downgrade(request));
        self.wake.notify_one();
        Ok(())
    }

    pub(super) fn finish(&self) {
        self.running.store(false, Ordering::Release);
        let (pending, failure) = match self.state.lock() {
            Ok(mut state) => (state.drain(), CanvasRequestError::WorkerStopped),
            Err(poisoned) => (
                poisoned.into_inner().drain(),
                CanvasRequestError::PoolPoisoned,
            ),
        };
        self.wake.notify_all();
        for request in pending.into_iter().filter_map(|request| request.upgrade()) {
            if let Err(error) = request.deliver(Err(failure.clone())) {
                tracing::error!(?error, "Canvas stopped-worker notification failed");
            }
        }
    }

    pub(super) fn release(&self) -> Result<(), CanvasRequestError> {
        self.state
            .lock()
            .map_err(|_| CanvasRequestError::PoolPoisoned)?
            .active = Weak::new();
        self.wake.notify_all();
        Ok(())
    }

    fn shutdown(&self) {
        let guard = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => {
                tracing::error!("Canvas queue was poisoned during shutdown");
                poisoned.into_inner()
            }
        };
        /* WHY: 待機ロック内で停止状態を更新し、通知と Condvar 待機のすれ違いを防ぐ。 */
        self.stopping.store(true, Ordering::Release);
        drop(guard);
        self.wake.notify_all();
    }
}
