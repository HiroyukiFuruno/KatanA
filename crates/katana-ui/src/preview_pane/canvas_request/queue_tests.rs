use std::sync::Arc;

use super::tests::{RequestTest, TEST_TIMEOUT};
use super::{
    CanvasParseRequest, CanvasRequestError,
    pool::{CanvasPool, QUEUE_LIMIT},
    state::RequestState,
};

#[test]
fn real_worker_queue_is_bounded_and_dropped_requests_free_admission() {
    let pool = CanvasPool::start().expect("worker start");
    let admitted = RequestTest::blocked(&pool, || {
        let mut requests: Vec<_> = (0..QUEUE_LIMIT)
            .map(|generation| RequestTest::submit(&pool, generation as u64, "{}"))
            .collect();
        assert_eq!(
            CanvasParseRequest::submit(&pool, 100, "{}".to_owned(), None).err(),
            Some(CanvasRequestError::QueueFull)
        );
        let weak = Arc::downgrade(&requests[0].request);
        drop(requests.remove(0));
        assert!(weak.upgrade().is_none());
        let admitted = RequestTest::submit(&pool, 101, "{}");
        assert_eq!(
            pool.shared.state.lock().expect("queue").pending.len(),
            QUEUE_LIMIT
        );
        admitted
    });
    assert_eq!(RequestTest::receive(&admitted).generation, 101);
}

#[test]
fn queued_cancelled_source_is_skipped_before_parser_runs() {
    let pool = CanvasPool::start().expect("worker start");
    let (cancelled, barrier) = RequestTest::blocked(&pool, || {
        let cancelled = RequestTest::submit(&pool, 11, "{invalid");
        assert!(cancelled.poll().expect("pending result").is_none());
        cancelled.cancel();
        assert_eq!(cancelled.poll().err(), Some(CanvasRequestError::Cancelled));
        let barrier = RequestTest::submit(&pool, 12, "{}");
        (cancelled, barrier)
    });
    RequestTest::receive(&barrier)
        .result
        .expect("barrier parsed");
    assert!(
        cancelled
            .request
            .source
            .lock()
            .expect("cancelled source")
            .is_some()
    );
}

#[test]
fn pool_mutex_poison_is_reported_and_pending_generations_wake_with_error() {
    let pool = CanvasPool::start().expect("worker start");
    let pending = RequestTest::blocked(&pool, || {
        let pending = RequestTest::submit(&pool, 99, "{}");
        let shared = Arc::clone(&pool.shared);
        let thread = std::thread::spawn(move || {
            let _guard = shared.state.lock().expect("queue lock");
            panic!("real pool-lock poison");
        });
        assert!(thread.join().is_err());
        assert_eq!(pending.poll().err(), Some(CanvasRequestError::PoolPoisoned));
        pending
    });
    let result = RequestTest::receive(&pending);
    assert_eq!(result.generation, 99);
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::PoolPoisoned)
    ));
}

#[test]
fn dropping_pool_completes_pending_request_with_worker_stopped_error() {
    let pool = CanvasPool::start().expect("worker start");
    let (blocker, receiver) = RequestState::new(0, "{}".to_owned(), None);
    let guard = blocker.source.lock().expect("source gate");
    pool.shared.enqueue(&blocker).expect("blocking request");
    RequestTest::wait_active(&pool, &blocker);
    let pending = RequestTest::submit(&pool, 5, "{}");
    let stopper = RequestTest::stop_pool(pool);
    drop(guard);
    receiver
        .recv_timeout(TEST_TIMEOUT)
        .expect("blocker completes");
    let result = RequestTest::receive(&pending);
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::WorkerStopped)
    ));
    stopper.join().expect("pool shutdown");
}
