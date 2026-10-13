use std::sync::Arc;
use std::time::Duration;

use super::{
    CanvasParseRequest, CanvasRequestError, pool::CanvasPool, state::RequestState,
    worker::CanvasWorker,
};

pub(super) const TEST_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) struct RequestTest;

impl RequestTest {
    pub(super) fn source(content: &str) -> String {
        let directory = tempfile::tempdir().expect("temporary source");
        let path = directory.path().join("source.canvas");
        std::fs::write(&path, content).expect("source write");
        std::fs::read_to_string(path).expect("host source read")
    }

    pub(super) fn submit(pool: &CanvasPool, generation: u64, source: &str) -> CanvasParseRequest {
        CanvasParseRequest::submit(pool, generation, Self::source(source), None).expect("submit")
    }

    pub(super) fn receive(request: &CanvasParseRequest) -> super::CanvasParseResult {
        request
            .receiver
            .recv_timeout(TEST_TIMEOUT)
            .expect("worker response")
    }

    pub(super) fn wait_active(pool: &CanvasPool, request: &Arc<RequestState>) {
        let state = pool.shared.state.lock().expect("queue lock");
        let (_state, timeout) = pool
            .shared
            .wake
            .wait_timeout_while(state, TEST_TIMEOUT, |state| {
                state.active.as_ptr() != Arc::as_ptr(request)
            })
            .expect("worker condition");
        assert!(!timeout.timed_out(), "worker did not start request");
    }

    pub(super) fn blocked<T>(pool: &CanvasPool, task: impl FnOnce() -> T) -> T {
        let (blocker, receiver) = RequestState::new(0, "{}".to_owned(), None);
        let guard = blocker.source.lock().expect("source gate");
        pool.shared.enqueue(&blocker).expect("blocking request");
        Self::wait_active(pool, &blocker);
        let result = task();
        drop(guard);
        receiver
            .recv_timeout(TEST_TIMEOUT)
            .expect("blocker completes");
        result
    }

    pub(super) fn stop_pool(pool: CanvasPool) -> std::thread::JoinHandle<()> {
        let shared = Arc::clone(&pool.shared);
        let stopper = std::thread::spawn(move || drop(pool));
        let state = shared.state.lock().expect("queue lock");
        let (_state, timeout) = shared
            .wake
            .wait_timeout_while(state, TEST_TIMEOUT, |_| {
                !shared.stopping.load(std::sync::atomic::Ordering::Acquire)
            })
            .expect("stop condition");
        assert!(!timeout.timed_out(), "pool did not begin shutdown");
        stopper
    }
}

#[test]
fn real_worker_parses_host_source_and_returns_exact_generation() {
    let source = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":100,"height":40,"text":"Canvas"}]}"#;
    let request =
        CanvasParseRequest::start(42, RequestTest::source(source), None).expect("request");
    let result = RequestTest::receive(&request);
    let document = result.result.expect("parsed model");
    assert_eq!(result.generation, 42);
    assert_eq!(document.nodes[0].id, "a");
    assert_eq!(Arc::strong_count(&document), 1);
}

#[test]
fn real_worker_reports_malformed_source_as_typed_failure() {
    let pool = CanvasPool::start().expect("worker start");
    let request = RequestTest::submit(&pool, 7, "{broken");
    let result = RequestTest::receive(&request);
    assert_eq!(result.generation, 7);
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::InvalidCanvas(_))
    ));
}

#[test]
fn repeated_global_access_uses_the_same_fixed_worker_pool() {
    let first = CanvasPool::global().expect("global pool");
    let second = CanvasPool::global().expect("global pool again");
    assert!(std::ptr::eq(first, second));
    assert!(Arc::ptr_eq(&first.shared, &second.shared));
}

#[test]
fn real_channel_disconnect_is_a_typed_delivery_error() {
    let (request, receiver) = RequestState::new(1, RequestTest::source("{}"), None);
    drop(receiver);
    assert_eq!(
        CanvasWorker::process(&request),
        Err(CanvasRequestError::ResultDisconnected)
    );
}

#[test]
fn missing_taken_source_is_not_silently_reparsed() {
    let (request, receiver) = RequestState::new(1, RequestTest::source("{}"), None);
    request.take_source().expect("first source take");
    CanvasWorker::process(&request).expect("typed result delivery");
    let result = receiver.recv_timeout(TEST_TIMEOUT).expect("response");
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::SourceUnavailable)
    ));
}

#[test]
fn request_source_mutex_poison_is_reported_by_real_worker() {
    let (request, receiver) = RequestState::new(3, RequestTest::source("{}"), None);
    let poisoned = Arc::clone(&request);
    let thread = std::thread::spawn(move || {
        let _guard = poisoned.source.lock().expect("source lock");
        panic!("real request-lock poison");
    });
    assert!(thread.join().is_err());
    let pool = CanvasPool::start().expect("worker start");
    pool.shared
        .enqueue(&request)
        .expect("enqueue poisoned request");
    let result = receiver.recv_timeout(TEST_TIMEOUT).expect("response");
    assert_eq!(result.generation, 3);
    assert!(matches!(
        result.result,
        Err(CanvasRequestError::RequestPoisoned)
    ));
}
