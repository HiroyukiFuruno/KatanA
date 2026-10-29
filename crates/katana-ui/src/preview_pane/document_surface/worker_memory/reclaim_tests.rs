use super::*;
use std::time::{Duration, Instant};

const CLEANUP_DEADLINE: Duration = Duration::from_secs(5);

fn await_cleanup(state: &WorkerMemoryState) {
    let deadline = Instant::now() + CLEANUP_DEADLINE;
    while state.cleanup_running() {
        assert!(
            Instant::now() < deadline,
            "background cleanup did not finish"
        );
        std::thread::yield_now();
    }
}

#[test]
fn idle_cleanup_is_scheduled_and_finishes_without_retaining_state_lock() {
    let state = Arc::new(WorkerMemoryState::new());
    state.request_relief();
    state.cancel_relief();
    state.acquire();
    state.release();
    await_cleanup(&state);
    assert!(state.relief_pending.try_lock().is_ok());
    assert_eq!(state.counter.load(Ordering::Acquire), 0);
    assert!(!*state.pending());
}

#[test]
fn newer_document_activation_rejects_a_stale_cleanup_claim() {
    let state = Arc::new(WorkerMemoryState::new());
    let epoch = state.relief_epoch.load(Ordering::Acquire);
    state.acquire();
    state.release();
    assert!(!state.claim_relief(epoch));
    assert!(!*state.pending());
}

#[test]
fn intake_arriving_before_cleanup_preserves_the_deferred_request() {
    let state = Arc::new(WorkerMemoryState::new());
    let epoch = state.relief_epoch.load(Ordering::Acquire);
    let intake = state.retain_intake();
    assert!(!state.claim_relief(epoch));
    assert!(*state.pending());
    assert!(!state.cleanup_running());
    drop(intake);
    await_cleanup(&state);
    assert!(!*state.pending());
}

#[test]
fn a_busy_cleanup_worker_keeps_a_later_idle_request_for_its_next_pass() {
    let state = Arc::new(WorkerMemoryState::new());
    state.relief_running.store(true, Ordering::Release);
    state.request_relief();
    assert!(*state.pending());
    assert!(state.next_relief().is_some());
    assert!(!*state.pending());
    assert!(state.next_relief().is_none());
    assert!(!state.cleanup_running());
}

#[test]
fn observer_does_not_report_idle_during_worker_to_cleanup_handoff() {
    let state = Arc::new(WorkerMemoryState::new());
    let pending = state.pending();
    let observed = Arc::clone(&state);
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (count_tx, count_rx) = std::sync::mpsc::channel();
    let observer = std::thread::spawn(move || {
        started_tx.send(()).expect("observer started");
        count_tx.send(observed.activity_count()).expect("activity");
    });
    started_rx.recv().expect("observer started");
    assert!(matches!(
        count_rx.recv_timeout(Duration::from_millis(50)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));
    state.relief_running.store(true, Ordering::Release);
    drop(pending);
    assert_eq!(count_rx.recv_timeout(CLEANUP_DEADLINE).expect("count"), 1);
    observer.join().expect("observer completed");
    state.relief_running.store(false, Ordering::Release);
}
