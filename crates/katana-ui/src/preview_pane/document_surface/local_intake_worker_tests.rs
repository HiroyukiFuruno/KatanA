use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Condvar, Mutex};

use super::super::pool::{PoolPhase, PoolState, SharedPool};
use super::super::request::{IntakeRequest, IntakeResult};
use super::WorkerExitGuard;

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
#[cfg(unix)]
const FIFO_CANCEL_CHILD_ENV: &str = "KATANA_INTAKE_FIFO_CANCEL_CHILD";

#[cfg(unix)]
#[test]
fn cancelled_fifo_reads_release_workers_before_writer_eof() {
    use katana_core::system::ProcessService;

    let child_test = format!(
        "{}::cancelled_fifo_reads_child",
        module_path!().strip_prefix("katana_ui::").unwrap()
    );
    let executable = std::env::current_exe().unwrap();
    let output = ProcessService::create_command(executable.to_str().unwrap())
        .args(["--exact", &child_test, "--nocapture"])
        .env(FIFO_CANCEL_CHILD_ENV, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
}

#[cfg(unix)]
#[test]
fn cancelled_fifo_reads_child() {
    if std::env::var_os(FIFO_CANCEL_CHILD_ENV).is_none() {
        return;
    }

    use super::super::LocalDocumentIntake;
    use super::super::pool::IntakePool;
    use std::io::Write;

    let directory = tempfile::tempdir().unwrap();
    let fifo_paths = [
        directory.path().join("first.pdf"),
        directory.path().join("second.pdf"),
    ];
    for path in &fifo_paths {
        assert!(
            katana_core::system::ProcessService::create_command("mkfifo")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }

    let pending: Vec<_> = fifo_paths
        .iter()
        .map(|path| LocalDocumentIntake::start(path.clone(), false).unwrap())
        .collect();
    let deadline = std::time::Instant::now() + TIMEOUT;
    while !pending
        .iter()
        .all(|request| request.request.sender_is_taken())
    {
        assert!(
            std::time::Instant::now() < deadline,
            "workers did not start"
        );
        std::thread::yield_now();
    }

    /* WHY: writerのopen完了で実workerのFIFO openを確認し、取消前に読込開始を確定する。 */
    let mut writers: Vec<_> = fifo_paths
        .iter()
        .map(|path| std::fs::File::create(path).unwrap())
        .collect();
    drop(pending);

    for writer in &mut writers {
        if let Err(error) = writer.write_all(b"%PDF-1.7") {
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        }
    }

    let deadline = std::time::Instant::now() + TIMEOUT;
    loop {
        let active_count = IntakePool::global().shared.lock().unwrap().active.len();
        if active_count == 0 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "cancelled FIFO reads kept both workers until writer EOF"
        );
        std::thread::yield_now();
    }

    let regular = directory.path().join("unrelated.pdf");
    std::fs::write(&regular, b"%PDF-1.7").unwrap();
    let unrelated = LocalDocumentIntake::start(regular, false).unwrap();
    let mut source = unrelated.result.recv_timeout(TIMEOUT).unwrap().unwrap();
    assert!(source.take_bytes().unwrap().starts_with(b"%PDF-"));
    assert!(!unrelated.request.is_cancelled());

    drop(writers);
}

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
            active: Vec::new(),
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
