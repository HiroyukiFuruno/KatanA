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
const DEADLINE: Duration = Duration::from_secs(5);
const RECEIPT: &str = "font-worker-lifetime-verified";

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
    let worker = spawn_barrier_worker();
    let started = worker.started.recv_timeout(DEADLINE).unwrap();
    worker.cancelled.store(true, Ordering::Release);
    let cancelled_count = DocumentWorkerLifecycle::live_count();
    worker.release.send(()).unwrap();
    worker.thread.join().unwrap();
    let dropped = worker.dropped.recv_timeout(DEADLINE).unwrap();
    assert_eq!(started, (1, PAYLOAD_BYTES));
    assert_eq!(cancelled_count, 1);
    assert_eq!(dropped, (1, PAYLOAD_BYTES, true));
}

struct BarrierWorker {
    cancelled: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
    started: mpsc::Receiver<(usize, usize)>,
    release: mpsc::Sender<()>,
    dropped: mpsc::Receiver<(usize, usize, bool)>,
}

fn spawn_barrier_worker() -> BarrierWorker {
    let cancelled = Arc::new(AtomicBool::new(false));
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (dropped_tx, dropped_rx) = mpsc::channel();
    let worker_cancelled = Arc::clone(&cancelled);
    let worker = FontLookupWorker::spawn("font-lifecycle-regression".to_owned(), move || {
        let work = OwnedWork::new(worker_cancelled, dropped_tx);
        started_tx
            .send((DocumentWorkerLifecycle::live_count(), work.bytes.len()))
            .unwrap();
        release_rx.recv_timeout(DEADLINE).unwrap();
        drop(work);
    })
    .expect("font worker spawned");
    BarrierWorker {
        cancelled,
        thread: worker,
        started: started_rx,
        release: release_tx,
        dropped: dropped_rx,
    }
}
