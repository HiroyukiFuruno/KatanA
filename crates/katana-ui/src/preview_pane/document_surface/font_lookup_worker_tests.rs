use katana_core::system::ProcessService;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

use super::super::DocumentWorkerLifecycle;
use super::super::font_lookup_worker::FontLookupWorker;

const CHILD_ENV: &str = "KATANA_FONT_LOOKUP_WORKER_LIFECYCLE_CHILD";
const PAYLOAD_BYTES: usize = 4096;
const QUEUE_CAPACITY: usize = 8;
const DEADLINE: Duration = Duration::from_secs(5);
const RECEIPT: &str = "font-worker-lifetime-verified";

#[path = "font_lookup_worker_bounded_tests.rs"]
mod bounded_tests;

#[test]
fn product_font_lookup_worker_is_counted_until_cancelled_work_finishes() {
    let child_test = format!(
        "{}::font_lookup_worker_lifecycle_child",
        module_path!().strip_prefix("katana_ui::").unwrap()
    );
    let executable = std::env::current_exe().unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let receipt = temporary.path().join("lifecycle-receipt");
    let child = ProcessService::create_command(executable.to_str().unwrap())
        .args(["--exact", &child_test, "--nocapture"])
        .env(CHILD_ENV, &receipt)
        .spawn()
        .unwrap();
    let mut child = OwnedChild(child);
    assert!(child.wait_until_deadline().success());
    assert_eq!(std::fs::read_to_string(receipt).unwrap(), RECEIPT);
}

struct OwnedChild(std::process::Child);

impl OwnedChild {
    fn wait_until_deadline(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + DEADLINE;
        loop {
            if let Some(status) = self.0.try_wait().expect("child status") {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "child exceeded lifecycle deadline"
            );
            std::thread::yield_now();
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

#[test]
fn font_lookup_worker_lifecycle_child() {
    let Some(receipt) = std::env::var_os(CHILD_ENV) else {
        return;
    };

    let baseline = DocumentWorkerLifecycle::live_count();
    assert_eq!(baseline, 0);
    run_barrier_worker();
    bounded_tests::run_queue_bound();
    bounded_tests::run_panic_recovery();
    assert_eq!(DocumentWorkerLifecycle::live_count(), baseline);
    std::fs::write(receipt, RECEIPT).expect("verified child receipt");
}

struct OwnedWork {
    cancelled: Arc<AtomicBool>,
    bytes: Vec<u8>,
    dropped: mpsc::Sender<(usize, usize, bool)>,
}

impl OwnedWork {
    fn new(cancelled: Arc<AtomicBool>, dropped: mpsc::Sender<(usize, usize, bool)>) -> Self {
        Self {
            cancelled,
            bytes: vec![0; PAYLOAD_BYTES],
            dropped,
        }
    }
}

impl Drop for OwnedWork {
    fn drop(&mut self) {
        self.dropped
            .send((
                DocumentWorkerLifecycle::live_count(),
                self.bytes.len(),
                self.cancelled.load(Ordering::Acquire),
            ))
            .unwrap();
    }
}

fn run_barrier_worker() {
    let BarrierWorker {
        cancelled,
        first_started,
        first_dropped,
        release,
        second_started,
        second_dropped,
        second_started_tx,
        second_dropped_tx,
        other_started,
        other_dropped,
        other_started_tx,
        other_dropped_tx,
    } = spawn_barrier_worker();
    let started = first_started.recv_timeout(DEADLINE).unwrap();
    assert_eq!(started, (1, PAYLOAD_BYTES));
    cancelled.store(true, Ordering::Release);
    enqueue_payload(other_started_tx, other_dropped_tx);
    enqueue_payload(second_started_tx, second_dropped_tx);
    assert_eq!(DocumentWorkerLifecycle::live_count(), 1);
    release.send(()).unwrap();
    assert_eq!(
        first_dropped.recv_timeout(DEADLINE).unwrap(),
        (1, PAYLOAD_BYTES, true)
    );
    assert_eq!(
        other_started.recv_timeout(DEADLINE).unwrap(),
        (1, PAYLOAD_BYTES)
    );
    assert_eq!(
        other_dropped.recv_timeout(DEADLINE).unwrap(),
        (1, PAYLOAD_BYTES, false)
    );
    assert_eq!(
        second_started.recv_timeout(DEADLINE).unwrap(),
        (1, PAYLOAD_BYTES)
    );
    assert_eq!(
        second_dropped.recv_timeout(DEADLINE).unwrap(),
        (1, PAYLOAD_BYTES, false)
    );
    wait_for_idle();
}

struct BarrierWorker {
    cancelled: Arc<AtomicBool>,
    first_started: mpsc::Receiver<(usize, usize)>,
    first_dropped: mpsc::Receiver<(usize, usize, bool)>,
    release: mpsc::Sender<()>,
    second_started: mpsc::Receiver<(usize, usize)>,
    second_dropped: mpsc::Receiver<(usize, usize, bool)>,
    second_started_tx: mpsc::Sender<(usize, usize)>,
    second_dropped_tx: mpsc::Sender<(usize, usize, bool)>,
    other_started: mpsc::Receiver<(usize, usize)>,
    other_dropped: mpsc::Receiver<(usize, usize, bool)>,
    other_started_tx: mpsc::Sender<(usize, usize)>,
    other_dropped_tx: mpsc::Sender<(usize, usize, bool)>,
}

fn spawn_barrier_worker() -> BarrierWorker {
    let cancelled = Arc::new(AtomicBool::new(false));
    let (first_started_tx, first_started_rx) = mpsc::channel();
    let (second_started_tx, second_started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (first_dropped_tx, first_dropped_rx) = mpsc::channel();
    let (second_dropped_tx, second_dropped_rx) = mpsc::channel();
    let (other_started_tx, other_started_rx) = mpsc::channel();
    let (other_dropped_tx, other_dropped_rx) = mpsc::channel();
    let worker_cancelled = Arc::clone(&cancelled);
    FontLookupWorker::enqueue(
        Arc::clone(&worker_cancelled),
        Box::new(move || {
            let work = OwnedWork::new(worker_cancelled, first_dropped_tx);
            first_started_tx
                .send((DocumentWorkerLifecycle::live_count(), work.bytes.len()))
                .unwrap();
            release_rx.recv_timeout(DEADLINE).unwrap();
            drop(work);
        }),
    )
    .expect("font worker spawned");
    BarrierWorker {
        cancelled,
        first_started: first_started_rx,
        first_dropped: first_dropped_rx,
        release: release_tx,
        second_started: second_started_rx,
        second_dropped: second_dropped_rx,
        second_started_tx,
        second_dropped_tx,
        other_started: other_started_rx,
        other_dropped: other_dropped_rx,
        other_started_tx,
        other_dropped_tx,
    }
}

fn enqueue_payload(
    started: mpsc::Sender<(usize, usize)>,
    dropped: mpsc::Sender<(usize, usize, bool)>,
) {
    let cancelled = Arc::new(AtomicBool::new(false));
    FontLookupWorker::enqueue(
        Arc::clone(&cancelled),
        Box::new(move || {
            let work = OwnedWork::new(cancelled, dropped);
            started
                .send((DocumentWorkerLifecycle::live_count(), work.bytes.len()))
                .unwrap();
            drop(work);
        }),
    )
    .unwrap();
}

fn wait_for_idle() {
    let deadline = Instant::now() + DEADLINE;
    while DocumentWorkerLifecycle::live_count() != 0 {
        assert!(Instant::now() < deadline, "font worker did not become idle");
        std::thread::yield_now();
    }
}
