use super::super::source::DocumentSurfaceSource;
use super::super::source_revision::{IntakeOutcome, SourceRevision};
use super::{
    LocalDocumentIntake, PreviewPane,
    pool::IntakePool,
    request::{IntakeRequest, IntakeResult},
};
use katana_core::system::ProcessService;
use std::sync::Arc;

const CHILD_ENV: &str = "KATANA_INTAKE_REVISION_CHILD";
const RESULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

fn run_isolated(child_test_name: &str) -> bool {
    if std::env::var_os(CHILD_ENV).is_some() {
        return false;
    }

    let child_test = format!(
        "{}::{child_test_name}",
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
    true
}

fn receive_result_and_drain(pending: &LocalDocumentIntake) -> IntakeResult {
    let result = pending
        .result
        .recv_timeout(RESULT_TIMEOUT)
        .expect("document intake result within deadline");
    wait_for_inactive_request(&pending.request);
    result
}

fn receive_unchanged_and_drain(
    pending: &LocalDocumentIntake,
    expected: &SourceRevision,
) -> IntakeResult {
    let result = receive_result_and_drain(pending);
    assert!(
        matches!(&result, Ok(IntakeOutcome::Unchanged(actual)) if actual == expected),
        "expected unchanged revision {expected:?}, got {result:?}"
    );
    result
}

fn wait_for_inactive_request(request: &Arc<IntakeRequest>) {
    let deadline = std::time::Instant::now() + RESULT_TIMEOUT;
    loop {
        let request_is_active = IntakePool::global()
            .shared
            .lock()
            .unwrap()
            .active
            .iter()
            .any(|active| {
                active
                    .upgrade()
                    .is_some_and(|active| Arc::ptr_eq(&active, request))
            });
        if !request_is_active {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "document intake worker did not release the completed request"
        );
        std::thread::yield_now();
    }
}

#[test]
fn manual_promotion_of_queued_unchanged_result_starts_forced_payload_read() {
    if run_isolated("manual_promotion_of_queued_unchanged_result_starts_forced_payload_read") {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("manual.pdf");
    std::fs::write(&path, b"%PDF-1.7\nmanual").unwrap();
    let source = DocumentSurfaceSource::local(&path).unwrap();
    let known = SourceRevision::from_source(&source, 1);
    let mut pending =
        LocalDocumentIntake::start_known(path.clone(), false, Some(known.clone())).unwrap();
    pending.force = true;
    let mut pane = PreviewPane::default();
    let result = receive_unchanged_and_drain(&pending, &known);
    pane.finish_document_intake(pending, result);
    let restarted = pane.document_intake.as_ref().unwrap();
    assert!(restarted.force);
    assert!(restarted.request.known_revision.is_none());
    let restarted = pane.document_intake.take().unwrap();
    let result = receive_result_and_drain(&restarted);
    assert!(matches!(result, Ok(IntakeOutcome::Loaded(_))));
}

fn loaded_pane(path: &std::path::Path) -> PreviewPane {
    std::fs::write(
        path,
        include_bytes!(
            "../../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
        ),
    )
    .unwrap();
    let mut pane = PreviewPane::default();
    pane.full_render_document_source(DocumentSurfaceSource::local(path).unwrap(), false);
    pane
}

#[test]
fn unchanged_result_preserves_matching_generation_and_queued_recheck() {
    if run_isolated("unchanged_result_preserves_matching_generation_and_queued_recheck") {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("same.pdf");
    let mut pane = loaded_pane(&path);
    let known = SourceRevision::from_surface(pane.document_surface.as_ref().unwrap());
    let pending =
        LocalDocumentIntake::start_known(path.clone(), false, Some(known.clone())).unwrap();
    let result = receive_unchanged_and_drain(&pending, &known);
    pane.finish_document_intake(pending, result);
    assert!(known.matches(pane.document_surface.as_ref()));
    assert!(pane.document_intake.is_none());
    let mut pending = LocalDocumentIntake::start_known(path, false, Some(known.clone())).unwrap();
    pending.refresh_requested = true;
    let result = receive_unchanged_and_drain(&pending, &known);
    pane.finish_document_intake(pending, result);
    assert_eq!(
        pane.document_intake
            .as_ref()
            .unwrap()
            .request
            .known_revision,
        Some(known.clone())
    );
    let recheck = pane.document_intake.take().unwrap();
    let result = receive_unchanged_and_drain(&recheck, &known);
    pane.finish_document_intake(recheck, result);
    assert!(pane.document_intake.is_none());
}

#[test]
fn queued_unchanged_result_cannot_replace_a_newer_surface_generation() {
    if run_isolated("queued_unchanged_result_cannot_replace_a_newer_surface_generation") {
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("generation.pdf");
    let mut pane = loaded_pane(&path);
    let previous = SourceRevision::from_surface(pane.document_surface.as_ref().unwrap());
    let pending =
        LocalDocumentIntake::start_known(path.clone(), false, Some(previous.clone())).unwrap();
    pane.full_render_document_source(DocumentSurfaceSource::local(&path).unwrap(), true);
    let current = SourceRevision::from_surface(pane.document_surface.as_ref().unwrap());
    assert_ne!(previous, current);
    let result = receive_unchanged_and_drain(&pending, &previous);
    pane.finish_document_intake(pending, result);
    assert!(current.matches(pane.document_surface.as_ref()));
    assert_eq!(
        pane.document_intake
            .as_ref()
            .unwrap()
            .request
            .known_revision,
        Some(current.clone())
    );
    let recheck = pane.document_intake.take().unwrap();
    let result = receive_unchanged_and_drain(&recheck, &current);
    pane.finish_document_intake(recheck, result);
    assert!(pane.document_intake.is_none());
    assert!(current.matches(pane.document_surface.as_ref()));
}
