use super::HtmlRuntimeTestGuard;
use crate::preview_pane::DocumentWorkerLifecycle;

#[test]
fn unrelated_document_worker_does_not_block_html_environment_release() {
    let (release, wait) = std::sync::mpsc::channel();
    let worker = DocumentWorkerLifecycle::spawn("unrelated-document".to_owned(), move || {
        wait.recv().expect("release unrelated worker");
    })
    .expect("start unrelated worker");
    let result = std::panic::catch_unwind(|| drop(HtmlRuntimeTestGuard::lock()));
    release.send(()).expect("release unrelated worker");
    worker.join().expect("join unrelated worker");
    assert!(result.is_ok(), "HTML guard waited for an unrelated worker");
}

#[test]
fn owned_html_cleanup_completion_is_joined_before_environment_release() {
    let result = std::panic::catch_unwind(|| {
        let guard = HtmlRuntimeTestGuard::lock();
        let worker = std::thread::spawn(|| panic!("owned cleanup panic sentinel"));
        HtmlRuntimeTestGuard::observe(worker);
        assert_eq!(guard.workers.0.borrow().len(), 1);
        drop(guard);
    });
    assert!(result.is_err(), "owned worker failure was not collected");
}

#[test]
fn real_html_pane_drop_registers_and_joins_its_cleanup_worker() {
    use katana_document_viewer::browser_session::{HtmlBrowserSource, HtmlBrowserViewport};

    let guard = HtmlRuntimeTestGuard::lock();
    let mut pane = crate::preview_pane::PreviewPane::default();
    let source = HtmlBrowserSource::new("<h1>Owned close</h1>", "https://example.test/owned")
        .expect("HTML source");
    pane.full_render_html_source(source, true);
    assert!(
        pane.start_html_browser_for_test(
            HtmlBrowserViewport::new(320, 240, 1.0).expect("viewport")
        )
    );
    drop(pane);
    assert_eq!(guard.workers.0.borrow().len(), 1);
    let observed = std::rc::Rc::clone(&guard.workers);
    drop(guard);
    assert!(observed.0.borrow().is_empty());
}
