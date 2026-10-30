use crate::test_render_env::RenderEnvLock;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::MutexGuard;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

#[path = "runtime_test_guard_tests.rs"]
mod tests;

thread_local! {
    static ACTIVE_CLOSE_WORKERS: RefCell<Option<Rc<CloseWorkers>>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct CloseWorkers(RefCell<Vec<JoinHandle<()>>>);

pub(crate) struct HtmlRuntimeTestGuard {
    _environment: MutexGuard<'static, ()>,
    workers: Rc<CloseWorkers>,
}

impl HtmlRuntimeTestGuard {
    pub(crate) fn lock() -> Self {
        let environment = RenderEnvLock::lock();
        let workers = Rc::new(CloseWorkers::default());
        ACTIVE_CLOSE_WORKERS.with(|active| *active.borrow_mut() = Some(Rc::clone(&workers)));
        Self {
            _environment: environment,
            workers,
        }
    }

    pub(in crate::preview_pane) fn observe(worker: JoinHandle<()>) {
        ACTIVE_CLOSE_WORKERS.with(|active| {
            if let Some(workers) = active.borrow().as_ref() {
                workers.0.borrow_mut().push(worker);
            }
        });
    }
}

impl Drop for HtmlRuntimeTestGuard {
    fn drop(&mut self) {
        ACTIVE_CLOSE_WORKERS.with(|active| active.borrow_mut().take());
        /* WHY: この環境のHTML終了だけを待ち、別テストのOfficeやallocatorを混入させない。 */
        self.workers.finish();
    }
}

impl CloseWorkers {
    fn all_finished(&self) -> bool {
        self.0.borrow().iter().all(JoinHandle::is_finished)
    }

    fn finish(&self) {
        let deadline = Instant::now() + CLOSE_TIMEOUT;
        while !self.all_finished() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        if !std::thread::panicking() {
            assert!(
                self.all_finished(),
                "owned HTML cleanup exceeded the existing five-second close budget"
            );
            for worker in self.0.take() {
                worker.join().expect("owned HTML cleanup worker panicked");
            }
        }
    }
}
