use super::super::request::IntakeRequest;
use super::{IntakePool, PoolPhase, PoolState, SharedPool, WORKER_LIMIT};
use std::sync::{Arc, Condvar, Mutex};

#[test]
fn cancelled_active_capacity_is_deterministic_and_recovers_after_release() {
    let (cancelled, _) = IntakeRequest::new("cancelled.pdf".into());
    let (active, _) = IntakeRequest::new("active.pdf".into());
    let (queued, _) = IntakeRequest::new("queued.pdf".into());
    let (later, _) = IntakeRequest::new("later.pdf".into());
    let shared = Arc::new(SharedPool {
        state: Mutex::new(PoolState {
            phase: PoolPhase::Ready,
            pending: std::collections::VecDeque::new(),
            active: vec![Arc::downgrade(&cancelled), Arc::downgrade(&active)],
        }),
        wake: Condvar::new(),
    });
    let pool = IntakePool {
        shared,
        _workers: Vec::with_capacity(WORKER_LIMIT),
    };

    assert!(pool.capacity_failure_cause(&cancelled).is_none());
    assert!(pool.capacity_failure_cause(&active).is_none());
    assert!(pool.capacity_failure_cause(&queued).is_none());
    assert!(pool.enqueue(&queued).is_ok());

    cancelled.cancel();
    let queued_cause = pool
        .capacity_failure_cause(&queued)
        .expect("queued request must see cancelled active capacity");
    assert_eq!(
        queued_cause,
        "document intake capacity is occupied by cancelled reads (worker limit 2)"
    );
    assert_eq!(pool.enqueue(&later), Err(queued_cause.clone()));
    assert!(pool.capacity_failure_cause(&cancelled).is_none());
    assert!(pool.capacity_failure_cause(&active).is_none());

    /* WHY: 取消済みrequestを残し、空き枠が戻ったこと自体で再受理できると確認する。 */
    pool.shared.release(&active);
    assert!(pool.capacity_failure_cause(&later).is_none());
    assert!(pool.enqueue(&later).is_ok());
}
