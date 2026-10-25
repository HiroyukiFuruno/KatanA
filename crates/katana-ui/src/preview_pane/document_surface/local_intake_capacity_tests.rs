use super::{LocalDocumentIntake, pool::IntakePool};
use crate::preview_pane::PreviewPane;
use katana_core::system::ProcessService;
use std::os::unix::fs::OpenOptionsExt;

const CHILD_ENV: &str = "KATANA_INTAKE_CAPACITY_CHILD";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[test]
fn cancelled_fifo_reads_terminate_pending_requests_and_recover_capacity() {
    let child_test = format!(
        "{}::fifo_capacity_child",
        module_path!().strip_prefix("katana_ui::").unwrap()
    );
    let executable = std::env::current_exe().unwrap();
    let stdout = tempfile::NamedTempFile::new().unwrap();
    let stderr = tempfile::NamedTempFile::new().unwrap();
    /* WHY: pipe満杯による待機を避け、子の停止期限まで出力を保持する。 */
    let child = ProcessService::create_command(executable.to_str().unwrap())
        .args(["--exact", &child_test, "--nocapture"])
        .env(CHILD_ENV, "1")
        .stdout(stdout.reopen().unwrap())
        .stderr(stderr.reopen().unwrap())
        .spawn()
        .unwrap();
    let mut child = CapacityChild(child);
    let status = child.wait_until_deadline();
    let stdout = std::fs::read_to_string(stdout.path()).unwrap();
    let stderr = std::fs::read_to_string(stderr.path()).unwrap();
    assert!(
        status.is_some_and(|status| status.success()),
        "capacity child failed or exceeded deadline: stdout={stdout} stderr={stderr}"
    );
    assert!(stdout.contains("running 1 test"));
}

struct CapacityChild(std::process::Child);

impl CapacityChild {
    fn wait_until_deadline(&mut self) -> Option<std::process::ExitStatus> {
        let deadline = std::time::Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.0.try_wait().expect("capacity child status") {
                return Some(status);
            }
            if std::time::Instant::now() >= deadline {
                self.0.kill().expect("stop capacity child at deadline");
                self.0.wait().expect("reap capacity child at deadline");
                return None;
            }
            std::thread::yield_now();
        }
    }
}

impl Drop for CapacityChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

#[test]
fn fifo_capacity_child() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let regular = directory.path().join("regular.pdf");
    let fifos = create_inputs(directory.path(), &regular);
    let mut active = occupy_reads(&fifos);
    /* WHY: sender取得だけでは取消前にFIFOのopenへ到達したことを保証できない。 */
    let writers: Vec<_> = fifos
        .iter()
        .map(|fifo| wait_for_fifo_reader(fifo, std::time::Instant::now() + TIMEOUT).unwrap())
        .collect();
    let mut queued = PreviewPane::default();
    queued.full_render_document_path(&regular, false);
    assert!(queued.document_intake.is_some());
    drop(active.pop());
    assert!(
        IntakePool::global()
            .capacity_failure_cause(&active[0].request)
            .is_none()
    );
    queued.poll_document_intake(&eframe::egui::Context::default());
    assert_capacity_failure(&queued);
    let mut later = PreviewPane::default();
    later.full_render_document_path(&regular, false);
    assert_capacity_failure(&later);
    assert!(IntakePool::global().failure_cause().is_none());
    drop(active);
    /* WHY: 取消前に接続したwriterを閉じ、read待機へEOFを通知する。 */
    drop(writers);
    wait_for_reads_to_finish();
    verify_recovered_read(&regular);
}

fn create_inputs(
    directory: &std::path::Path,
    regular: &std::path::Path,
) -> [std::path::PathBuf; 2] {
    let fifos = [directory.join("first.pdf"), directory.join("second.pdf")];
    for fifo in &fifos {
        assert!(
            ProcessService::create_command("mkfifo")
                .arg(fifo)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(
        regular,
        include_bytes!(
            "../../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
        ),
    )
    .unwrap();
    fifos
}

fn occupy_reads(fifos: &[std::path::PathBuf]) -> Vec<LocalDocumentIntake> {
    let active: Vec<_> = fifos
        .iter()
        .map(|path| LocalDocumentIntake::start(path.clone(), false).unwrap())
        .collect();
    let deadline = std::time::Instant::now() + TIMEOUT;
    while !active
        .iter()
        .all(|pending| pending.request.sender_is_taken())
    {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    active
}

fn assert_capacity_failure(pane: &PreviewPane) {
    assert!(pane.document_intake.is_none());
    assert!(!pane.is_loading);
    let failure = pane
        .document_failure
        .as_ref()
        .expect("terminal capacity failure");
    assert_eq!(failure.operation, "start intake");
    assert!(failure.cause.contains("occupied by cancelled reads"));
}

fn wait_for_fifo_reader(
    fifo: &std::path::Path,
    deadline: std::time::Instant,
) -> std::io::Result<std::fs::File> {
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(fifo)
        {
            Ok(writer) => return Ok(writer),
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                if std::time::Instant::now() >= deadline {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("FIFO reader did not rendezvous: {}", fifo.display()),
                    ));
                }
                std::thread::yield_now();
            }
            Err(error) => return Err(error),
        }
    }
}

#[test]
fn cancellation_before_fifo_open_returns_a_bounded_rendezvous_failure() {
    let directory = tempfile::tempdir().unwrap();
    let regular = directory.path().join("regular.pdf");
    let fifos = create_inputs(directory.path(), &regular);
    let cancellation =
        super::super::source_revision::SourceRevisionOps::load(&fifos[0], None, || true)
            .unwrap_err();
    assert!(cancellation.cause.contains("cancelled"));
    let error = wait_for_fifo_reader(&fifos[0], std::time::Instant::now()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    /* WHY: reader不在以外のI/O障害を待機扱いして隠蔽しない。 */
    let error = wait_for_fifo_reader(
        &directory.path().join("missing.pdf"),
        std::time::Instant::now(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

fn wait_for_reads_to_finish() {
    let deadline = std::time::Instant::now() + TIMEOUT;
    while !IntakePool::global()
        .shared
        .lock()
        .unwrap()
        .active
        .is_empty()
    {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}

fn verify_recovered_read(regular: &std::path::Path) {
    let pending = LocalDocumentIntake::start(regular.to_path_buf(), false).unwrap();
    let super::super::source_revision::IntakeOutcome::Loaded(source) = pending
        .result
        .recv_timeout(TIMEOUT)
        .unwrap()
        .expect("recovered regular read")
    else {
        panic!("recovered request must load source")
    };
    assert_eq!(
        source.descriptor().uri,
        url::Url::from_file_path(regular.canonicalize().unwrap())
            .unwrap()
            .to_string()
    );
}
