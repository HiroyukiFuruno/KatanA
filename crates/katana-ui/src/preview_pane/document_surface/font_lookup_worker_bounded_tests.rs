use super::super::FontLookupWorker;
use super::{DEADLINE, OwnedWork, QUEUE_CAPACITY, wait_for_idle};
use std::sync::{Arc, atomic::AtomicBool, mpsc};

pub(super) fn run_queue_bound() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let blocker = Arc::new(AtomicBool::new(false));
    FontLookupWorker::enqueue(
        Arc::clone(&blocker),
        Box::new(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(DEADLINE).unwrap();
        }),
    )
    .unwrap();
    started_rx.recv_timeout(DEADLINE).unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let (dropped_tx, dropped_rx) = mpsc::channel();
    enqueue_owned(Arc::clone(&cancelled), dropped_tx);
    cancelled.store(true, std::sync::atomic::Ordering::Release);
    FontLookupWorker::enqueue(Arc::new(AtomicBool::new(false)), Box::new(|| {})).unwrap();
    assert_eq!(
        dropped_rx.recv_timeout(DEADLINE).unwrap(),
        (1, super::PAYLOAD_BYTES, true)
    );
    for _ in 0..QUEUE_CAPACITY - 1 {
        FontLookupWorker::enqueue(Arc::new(AtomicBool::new(false)), Box::new(|| {})).unwrap();
    }
    let overflow = FontLookupWorker::enqueue(Arc::new(AtomicBool::new(false)), Box::new(|| {}));
    assert_eq!(overflow.unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    release_tx.send(()).unwrap();
    wait_for_idle();
}

pub(super) fn run_panic_recovery() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (dropped_tx, dropped_rx) = mpsc::channel();
    FontLookupWorker::enqueue(
        Arc::new(AtomicBool::new(false)),
        Box::new(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(DEADLINE).unwrap();
            panic!("font worker regression panic");
        }),
    )
    .unwrap();
    started_rx.recv_timeout(DEADLINE).unwrap();
    enqueue_owned(Arc::new(AtomicBool::new(false)), dropped_tx);
    release_tx.send(()).unwrap();
    assert_eq!(
        dropped_rx.recv_timeout(DEADLINE).unwrap(),
        (1, super::PAYLOAD_BYTES, false)
    );
    wait_for_idle();
    let (done_tx, done_rx) = mpsc::channel();
    FontLookupWorker::enqueue(
        Arc::new(AtomicBool::new(false)),
        Box::new(move || done_tx.send(()).unwrap()),
    )
    .unwrap();
    done_rx.recv_timeout(DEADLINE).unwrap();
    wait_for_idle();
}

fn enqueue_owned(cancelled: Arc<AtomicBool>, dropped: mpsc::Sender<(usize, usize, bool)>) {
    let work = OwnedWork::new(Arc::clone(&cancelled), dropped);
    FontLookupWorker::enqueue(cancelled, Box::new(move || drop(work))).unwrap();
}
