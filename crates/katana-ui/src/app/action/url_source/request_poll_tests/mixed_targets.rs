use super::{app, html, open_html_targets, request, source_body};
use std::path::PathBuf;

#[test]
fn unrelated_ready_target_progresses_behind_pending_url_bar_request() {
    let mut app = app();
    let target = PathBuf::from("ready.html");
    open_html_targets(&mut app, &target, &PathBuf::from("other.html"));
    let (_held, pending) = request("https://example.test/slow", None);
    let (sender, ready) = request("https://example.test/ready", Some(target.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    sender
        .send(Ok(html("https://example.test/ready", "ready")))
        .unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(
        source_body(&app, &target).as_deref(),
        Some("<html><body>ready</body></html>")
    );
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert!(app.state.url_tab.is_loading);
}

#[test]
fn unrelated_url_bar_result_progresses_behind_pending_existing_target() {
    let mut app = app();
    let target = PathBuf::from("slow.html");
    open_html_targets(&mut app, &target, &PathBuf::from("other.html"));
    let (_held, pending) = request("https://example.test/slow", Some(target));
    let (sender, ready) = request("https://example.test/ready", None);
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    sender
        .send(Ok(html("https://example.test/ready", "ready")))
        .unwrap();
    app.poll_url_source(&egui::Context::default());
    let opened = super::super::document::remote_document_path("https://example.test/ready", "html");
    assert_eq!(
        source_body(&app, &opened).as_deref(),
        Some("<html><body>ready</body></html>")
    );
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert!(app.state.url_tab.is_loading);
}

#[test]
fn identical_source_preserves_order_across_mixed_targets() {
    let mut app = app();
    let url = "https://example.test/shared";
    let target = super::super::document::remote_document_path(url, "html");
    open_html_targets(&mut app, &target, &PathBuf::from("other.html"));
    let (first, pending) = request(url, None);
    let (second, ready) = request(url, Some(target.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    second.send(Ok(html(url, "second"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 2);
    first.send(Ok(html(url, "first"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(
        source_body(&app, &target).as_deref(),
        Some("<html><body>second</body></html>")
    );
    assert!(app.state.url_tab.pending_url_requests.is_empty());
}
