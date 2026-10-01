use super::LocalDocumentIntake;
use crate::preview_pane::PreviewPane;
use katana_core::system::ProcessService;

const CHILD_ENV: &str = "KATANA_INTAKE_RESOURCE_CHILD";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const CANCEL_ROUNDS: usize = 12;
const QUEUED_REQUEST_LIMIT: usize = 64;

#[test]
fn abandoned_fifo_reads_have_a_process_wide_worker_bound() {
    let child_test = format!(
        "{}::fifo_resource_child",
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
fn fifo_resource_child() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("blocked.pdf");
    assert!(
        ProcessService::create_command("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    let baseline = process_threads();
    let mut active_pane = active_fifo_pane(&path);
    abandon_reads(&path);
    let actual = wait_for_workers(baseline);
    println!("intake resource bound: baseline={baseline} actual={actual} max_workers=2");
    assert_eq!(actual, baseline + 2);
    verify_queue_bound(&path);
    verify_regular_switch(directory.path());
    assert_eq!(process_threads(), baseline + 2);
    verify_terminal_failure(&mut active_pane, &path);
}

fn active_fifo_pane(path: &std::path::Path) -> PreviewPane {
    let mut active = occupy_workers(path);
    let mut pane = PreviewPane::default();
    pane.document_intake = active.pop();
    pane.is_loading = true;
    pane
}

fn abandon_reads(path: &std::path::Path) {
    let mut pane = PreviewPane::default();
    for _ in 0..CANCEL_ROUNDS {
        pane.full_render_document_path(path, false);
        assert!(pane.document_intake.is_some());
        pane.document_intake = None;
    }
}

fn verify_terminal_failure(pane: &mut PreviewPane, path: &std::path::Path) {
    assert!(
        pane.document_intake
            .as_ref()
            .unwrap()
            .result
            .try_recv()
            .is_err()
    );
    super::pool::IntakePool::global()
        .shared
        .fail("worker stopped while OS read remains blocked");
    pane.poll_document_intake(&eframe::egui::Context::default());
    assert!(pane.document_intake.is_none());
    assert!(!pane.is_loading);
    assert!(
        pane.document_failure
            .as_ref()
            .unwrap()
            .cause
            .contains("worker stopped")
    );
    let failure = LocalDocumentIntake::start(path.to_path_buf(), false).unwrap_err();
    assert!(failure.cause.contains("worker stopped"));
}

fn occupy_workers(path: &std::path::Path) -> Vec<LocalDocumentIntake> {
    let active: Vec<_> = (0..2)
        .map(|_| LocalDocumentIntake::start(path.to_path_buf(), false).unwrap())
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

fn verify_queue_bound(path: &std::path::Path) {
    let pending: Vec<_> = (0..QUEUED_REQUEST_LIMIT)
        .map(|_| LocalDocumentIntake::start(path.to_path_buf(), false).unwrap())
        .collect();
    let failure = LocalDocumentIntake::start(path.to_path_buf(), false).unwrap_err();
    assert_eq!(failure.operation, "start intake");
    assert!(failure.cause.contains("queue is full (limit 64)"));
    let cancelled = std::sync::Arc::downgrade(&pending[0].request);
    drop(pending);
    assert!(cancelled.upgrade().is_none());
    let replacement = LocalDocumentIntake::start(path.to_path_buf(), false).unwrap();
    assert!(!replacement.request.sender_is_taken());
}

fn verify_regular_switch(directory: &std::path::Path) {
    let path = directory.join("regular.pdf");
    std::fs::write(&path, b"%PDF-1.7").unwrap();
    let mut pane = PreviewPane::default();
    pane.full_render_document_path(&path, false);
    let context = eframe::egui::Context::default();
    let mut frame = context.run_ui(eframe::egui::RawInput::default(), |ui| {
        pane.show_document_surface(ui);
    });
    assert!(!frame.shapes.is_empty());
    frame.textures_delta.clear();
    assert!(pane.document_intake.is_some());
    assert!(pane.document_failure.is_none());
    pane.document_intake = None;
    pane.poll_document_intake(&context);
    assert!(pane.document_surface.is_none());
}

fn wait_for_workers(baseline: usize) -> usize {
    let deadline = std::time::Instant::now() + TIMEOUT;
    loop {
        let actual = process_threads();
        if actual >= baseline + 2 {
            return actual;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}

#[cfg(target_os = "linux")]
fn process_threads() -> usize {
    std::fs::read_dir("/proc/self/task").unwrap().count()
}

#[cfg(not(target_os = "linux"))]
fn process_threads() -> usize {
    let output = ProcessService::create_command("ps")
        .args(["-M", "-p", &std::process::id().to_string(), "-o", "pid="])
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().lines().count()
}
