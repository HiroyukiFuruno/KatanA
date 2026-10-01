use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Condvar, Mutex};

use super::super::pool::{PoolPhase, PoolState, SharedPool};
use super::super::request::{IntakeRequest, IntakeResult};
use super::WorkerExitGuard;

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

struct FailureHarness {
    shared: Arc<SharedPool>,
    active: Arc<IntakeRequest>,
    active_result: Receiver<IntakeResult>,
    queued: Arc<IntakeRequest>,
    queued_result: Receiver<IntakeResult>,
}

impl FailureHarness {
    fn new() -> Self {
        let (active, active_result) = IntakeRequest::new("active.pdf".into());
        let (queued, queued_result) = IntakeRequest::new("queued.pdf".into());
        let state = PoolState {
            phase: PoolPhase::Ready,
            pending: VecDeque::from([Arc::downgrade(&queued)]),
        };
        Self {
            shared: Arc::new(SharedPool {
                state: Mutex::new(state),
                wake: Condvar::new(),
            }),
            active,
            active_result,
            queued,
            queued_result,
        }
    }
}

#[test]
fn worker_panic_disconnects_active_receiver_and_fails_queued_requests() {
    let harness = FailureHarness::new();
    let shared = harness.shared.clone();
    let request = harness.active.clone();
    let worker = std::thread::spawn(move || {
        let _guard = WorkerExitGuard(shared);
        let _sender = request.take_sender().unwrap().unwrap();
        panic!("real worker unwind");
    });
    assert!(worker.join().is_err());
    assert!(matches!(
        harness.active_result.recv_timeout(TIMEOUT),
        Err(RecvTimeoutError::Disconnected)
    ));
    let failure = harness
        .queued_result
        .recv_timeout(TIMEOUT)
        .unwrap()
        .unwrap_err();
    assert_eq!(failure.operation, "start intake");
    assert!(failure.cause.contains("worker stopped"));
    assert!(harness.queued.sender_is_taken());
    let state = harness.shared.lock().unwrap();
    assert!(matches!(state.phase, PoolPhase::Failed(_)));
    assert!(state.pending.is_empty());
}

#[test]
fn poisoned_queue_fails_queued_requests_without_success_recovery() {
    let harness = FailureHarness::new();
    let shared = harness.shared.clone();
    assert!(
        std::thread::spawn(move || {
            let _state = shared.state.lock().unwrap();
            panic!("real queue poison");
        })
        .join()
        .is_err()
    );
    assert!(harness.shared.lock().err().unwrap().contains("poisoned"));
    let failure = harness
        .queued_result
        .recv_timeout(TIMEOUT)
        .unwrap()
        .unwrap_err();
    assert!(failure.cause.contains("poisoned"));
    assert!(harness.queued.sender_is_taken());
}
