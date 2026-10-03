use katana_core::system::ProcessService;
use std::sync::{Arc, atomic::AtomicBool, mpsc};

use super::super::super::DocumentWorkerLifecycle;
use super::super::super::font_lookup_worker::FontLookupWorker;
use super::super::DocumentFontLookup;
use super::{
    RESULT_DEADLINE, RETRY_CHILD, RETRY_QUEUE_CAPACITY, RETRY_RECEIPT, SURFACE_GENERATION,
};
use crate::font_loader::office_faces::FontFaceRequest;

#[test]
fn document_lookup_retries_after_bounded_queue_recovers() {
    let child_test = format!(
        "{}::document_lookup_retry_child",
        module_path!().strip_prefix("katana_ui::").unwrap()
    );
    let executable = std::env::current_exe().unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let receipt = temporary.path().join("retry-receipt");
    let child = ProcessService::create_command(executable.to_str().unwrap())
        .args(["--exact", &child_test, "--nocapture"])
        .env(RETRY_CHILD, &receipt)
        .spawn()
        .unwrap();
    let mut child = RetryChild(child);
    assert!(child.wait_until_deadline().success());
    assert_eq!(std::fs::read_to_string(receipt).unwrap(), RETRY_RECEIPT);
}

struct RetryChild(std::process::Child);

impl RetryChild {
    fn wait_until_deadline(&mut self) -> std::process::ExitStatus {
        let deadline = std::time::Instant::now() + RESULT_DEADLINE;
        loop {
            if let Some(status) = self.0.try_wait().expect("retry child status") {
                return status;
            }
            assert!(std::time::Instant::now() < deadline, "retry child timeout");
            std::thread::yield_now();
        }
    }
}

impl Drop for RetryChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

#[test]
fn document_lookup_retry_child() {
    let Some(receipt) = std::env::var_os(RETRY_CHILD) else {
        return;
    };
    run_document_lookup_retry_child();
    std::fs::write(receipt, RETRY_RECEIPT).unwrap();
}

fn run_document_lookup_retry_child() {
    let (ctx, release_tx) = fill_retry_queue();
    let (family, _, _) = crate::font_loader::office_faces::installed_regular_bold_pair();
    let mut lookup = DocumentFontLookup::default();
    lookup.update(
        SURFACE_GENERATION,
        vec![FontFaceRequest {
            family: family.clone(),
            bold: true,
            italic: false,
        }],
        &ctx,
    );
    assert!(lookup.retry_pending && lookup.lease().is_some());
    release_tx.send(()).unwrap();
    poll_until_font_ready(&mut lookup, &ctx, &family);
    lookup.cancel();
    assert!(!lookup.retry_pending);
    wait_for_workers();
}

fn fill_retry_queue() -> (egui::Context, mpsc::Sender<()>) {
    let ctx = super::installed_context();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    FontLookupWorker::enqueue(
        Arc::new(AtomicBool::new(false)),
        Box::new(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(RESULT_DEADLINE).unwrap();
        }),
    )
    .unwrap();
    started_rx.recv_timeout(RESULT_DEADLINE).unwrap();
    for _ in 0..RETRY_QUEUE_CAPACITY {
        FontLookupWorker::enqueue(Arc::new(AtomicBool::new(false)), Box::new(|| {})).unwrap();
    }
    (ctx, release_tx)
}

fn poll_until_font_ready(lookup: &mut DocumentFontLookup, ctx: &egui::Context, family: &str) {
    let deadline = std::time::Instant::now() + RESULT_DEADLINE;
    while lookup.retry_pending || lookup.pending.is_some() {
        lookup.poll(SURFACE_GENERATION, ctx);
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(
        lookup
            .lease()
            .unwrap()
            .family_for(family, true, false)
            .is_some()
    );
}

fn wait_for_workers() {
    let deadline = std::time::Instant::now() + RESULT_DEADLINE;
    while DocumentWorkerLifecycle::live_count() != 0 {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}
