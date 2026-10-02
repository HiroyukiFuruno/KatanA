use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

const MAX_PENDING_FONT_LOOKUPS: usize = 8;

struct Work {
    cancelled: Arc<AtomicBool>,
    run: Box<dyn FnOnce() + Send + 'static>,
}

struct State {
    pending: VecDeque<Work>,
    running: bool,
}

static SHARED: OnceLock<Arc<Mutex<State>>> = OnceLock::new();

pub(super) struct FontLookupScheduler;

impl FontLookupScheduler {
    pub(super) fn enqueue(
        cancelled: Arc<AtomicBool>,
        run: Box<dyn FnOnce() + Send + 'static>,
    ) -> std::io::Result<()> {
        let shared = shared_state();
        let mut state = shared.lock().expect("font worker state");
        queue_work(&mut state, cancelled, run)?;
        if state.running {
            return Ok(());
        }
        state.running = true;
        match super::FontLookupWorker::spawn("katana-font-worker".to_owned(), {
            let shared = Arc::clone(&shared);
            move || run_worker(shared)
        }) {
            Ok(_) => Ok(()),
            Err(error) => {
                state.running = false;
                state.pending.clear();
                Err(error)
            }
        }
    }
}

fn shared_state() -> Arc<Mutex<State>> {
    SHARED
        .get_or_init(|| {
            Arc::new(Mutex::new(State {
                pending: VecDeque::new(),
                running: false,
            }))
        })
        .clone()
}

fn queue_work(
    state: &mut State,
    cancelled: Arc<AtomicBool>,
    run: Box<dyn FnOnce() + Send + 'static>,
) -> std::io::Result<()> {
    state
        .pending
        .retain(|work| !work.cancelled.load(Ordering::Acquire));
    if state.pending.len() >= MAX_PENDING_FONT_LOOKUPS {
        return Err(std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "font lookup worker queue is full",
        ));
    }
    state.pending.push_back(Work { cancelled, run });
    Ok(())
}

fn run_worker(shared: Arc<Mutex<State>>) {
    let mut guard = Guard {
        shared: Arc::clone(&shared),
        armed: true,
    };
    loop {
        let work = {
            let mut state = shared.lock().expect("font worker state");
            let Some(work) = state.pending.pop_front() else {
                guard.armed = false;
                state.running = false;
                return;
            };
            work
        };
        (work.run)();
    }
}

struct Guard {
    shared: Arc<Mutex<State>>,
    armed: bool,
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.armed {
            let mut state = self.shared.lock().expect("font worker state");
            state.running = false;
            state.pending.clear();
        }
    }
}
