use crate::preview_pane::PreviewPane;

const INTAKE_TEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[test]
fn missing_document_failure_is_delivered_asynchronously() {
    let directory = tempfile::tempdir().unwrap();
    let mut pane = PreviewPane::default();
    pane.full_render_document_path(&directory.path().join("missing.pdf"), false);
    assert!(pane.document_intake.is_some());
    assert!(pane.document_failure.is_none());
    assert_eq!(pane.document_is_idle_for_test(), Some(false));
    let context = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + INTAKE_TEST_TIMEOUT;
    while pane.document_intake.is_some() {
        assert!(std::time::Instant::now() < deadline);
        pane.poll_document_intake(&context);
        std::thread::yield_now();
    }
    assert_eq!(
        pane.document_failure.as_ref().unwrap().operation,
        "canonicalize"
    );
    assert!(!pane.is_loading);
}

#[test]
fn html_update_discards_a_pending_document_intake() {
    let directory = tempfile::tempdir().unwrap();
    let mut pane = PreviewPane::default();
    pane.full_render_document_path(&directory.path().join("missing.pdf"), false);
    pane.update_html_document_sections("<p>replacement</p>", &directory.path().join("next.html"));
    assert!(pane.document_intake.is_none());
    pane.poll_document_intake(&eframe::egui::Context::default());
    assert!(pane.document_failure.is_none());
    assert!(pane.html_browser.is_some());
}

#[cfg(unix)]
#[test]
fn blocked_file_read_does_not_block_ui_and_cancelled_result_cannot_replace_html() {
    use std::io::Write;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("blocked.pdf");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    let context = eframe::egui::Context::default();
    let mut pane = PreviewPane::default();
    pane.full_render_document_path(&path, false);
    pane.poll_document_intake(&context);
    assert!(pane.document_intake.is_some());
    let mut frame = context.run_ui(eframe::egui::RawInput::default(), |ui| {
        pane.show_document_surface(ui);
    });
    assert!(!frame.shapes.is_empty());
    frame.textures_delta.clear();
    assert!(pane.document_surface.is_none());
    assert!(pane.document_intake.is_some());
    let cancelled = pane.document_intake.take().unwrap();
    pane.full_render_html_document(
        "<p>new page</p>",
        &directory.path().join("next.html"),
        false,
    );
    assert!(pane.document_intake.is_none());
    let mut writer = std::fs::File::create(&path).unwrap();
    writer.write_all(b"%PDF-1.7").unwrap();
    drop(writer);
    assert!(
        cancelled
            .result
            .recv_timeout(INTAKE_TEST_TIMEOUT)
            .unwrap()
            .is_ok()
    );
    pane.poll_document_intake(&context);
    assert!(pane.document_surface.is_none());
    assert!(pane.html_browser.is_some());
}

fn finish_intake(pane: &mut PreviewPane, context: &eframe::egui::Context) {
    let deadline = std::time::Instant::now() + INTAKE_TEST_TIMEOUT;
    while pane.document_intake.is_some() {
        assert!(std::time::Instant::now() < deadline);
        pane.poll_document_intake(context);
        std::thread::yield_now();
    }
    assert!(pane.document_failure.is_none());
}

#[test]
fn unchanged_async_refresh_retains_session_but_changed_or_forced_refresh_replaces_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("report.pdf");
    let original = include_bytes!(
        "../../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
    );
    std::fs::write(&path, original).unwrap();
    let context = eframe::egui::Context::default();
    let mut pane = PreviewPane::default();
    pane.full_render_document_path(&path, false);
    finish_intake(&mut pane, &context);
    let generation = pane.document_surface.as_ref().unwrap().generation;
    let revision = pane
        .document_surface
        .as_ref()
        .unwrap()
        .source
        .revision
        .clone();
    pane.full_render_document_path(&path, false);
    finish_intake(&mut pane, &context);
    assert_eq!(
        pane.document_surface.as_ref().unwrap().generation,
        generation
    );
    let mut changed = original.to_vec();
    changed.extend_from_slice(b"\n");
    std::fs::write(&path, changed).unwrap();
    pane.full_render_document_path(&path, false);
    finish_intake(&mut pane, &context);
    let changed_surface = pane.document_surface.as_ref().unwrap();
    assert_ne!(changed_surface.generation, generation);
    assert_ne!(changed_surface.source.revision, revision);
    let changed_generation = changed_surface.generation;
    pane.full_render_document_path(&path, true);
    finish_intake(&mut pane, &context);
    assert_ne!(
        pane.document_surface.as_ref().unwrap().generation,
        changed_generation
    );
}

#[cfg(unix)]
#[test]
fn refresh_requests_during_a_read_coalesce_and_preserve_force() {
    use std::io::Write;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("blocked.xlsx");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    let mut pane = PreviewPane::default();
    pane.full_render_document_path(&path, false);
    pane.full_render_document_path(&path, true);
    pane.full_render_document_path(&path, false);
    let pending = pane.document_intake.take().unwrap();
    assert!(pending.force);
    assert!(pending.refresh_requested);
    let mut writer = std::fs::File::create(&path).unwrap();
    writer.write_all(b"PK\x03\x04").unwrap();
    drop(writer);
    assert!(
        pending
            .result
            .recv_timeout(INTAKE_TEST_TIMEOUT)
            .unwrap()
            .is_ok()
    );
}
