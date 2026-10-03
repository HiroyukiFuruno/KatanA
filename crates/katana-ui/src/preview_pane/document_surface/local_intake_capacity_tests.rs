use super::{LocalDocumentIntake, pool::IntakePool};
use crate::preview_pane::PreviewPane;
use katana_core::system::ProcessService;

const CHILD_ENV: &str = "KATANA_INTAKE_CAPACITY_CHILD";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[test]
fn cancelled_fifo_reads_terminate_pending_requests_and_recover_capacity() {
    let child_test = format!(
        "{}::fifo_capacity_child",
        module_path!().strip_prefix("katana_ui::").unwrap()
    );
    let executable = std::env::current_exe().unwrap();
    let output = ProcessService::create_command(executable.to_str().unwrap())
        .args(["--exact", &child_test, "--nocapture"])
        .env(CHILD_ENV, "1")
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

#[test]
fn fifo_capacity_child() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let regular = directory.path().join("regular.pdf");
    let fifos = create_inputs(directory.path(), &regular);
    let mut active = occupy_reads(&fifos);
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
    release_fifo_reads(&fifos);
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

fn release_fifo_reads(fifos: &[std::path::PathBuf]) {
    use std::io::Write;

    for fifo in fifos {
        let mut writer = std::fs::File::create(fifo).unwrap();
        writer.write_all(b"%PDF-1.7").unwrap();
        drop(writer);
    }
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
    let source = pending
        .result
        .recv_timeout(TIMEOUT)
        .unwrap()
        .expect("recovered regular read");
    assert_eq!(
        source.descriptor().uri,
        url::Url::from_file_path(regular.canonicalize().unwrap())
            .unwrap()
            .to_string()
    );
}
