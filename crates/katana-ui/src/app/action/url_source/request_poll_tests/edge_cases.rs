use super::{app, html, request, source_body};
use crate::state::HtmlSourceError;
use katana_core::document::Document;
use std::path::PathBuf;

const HTTP_FORBIDDEN: u16 = 403;

#[test]
fn closed_target_is_discarded_even_behind_pending_shared_input() {
    let mut app = app();
    let url = "https://example.test/source.html";
    let (held, pending) = request(url, None);
    let (ready, closed) = request(url, Some(PathBuf::from("closed.html")));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, closed]);
    ready.send(Ok(html(url, "closed"))).expect("ready response");
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert!(app.state.document.open_documents.is_empty());
    assert!(app.state.url_tab.is_loading);
    drop(held);
}

#[test]
fn redirected_shared_input_cannot_replace_newer_target() {
    let mut app = app();
    let url = "https://example.test/source.html";
    let target = super::super::document::remote_document_path(url, "html");
    app.state
        .document
        .open_documents
        .push(Document::new(target.clone(), "original"));
    let (first, pending) = request("https://example.test/alias", None);
    let (second, ready) = request(url, Some(target.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    second
        .send(Ok(html(url, "second")))
        .expect("second response");
    app.poll_url_source(&egui::Context::default());
    assert_target_body(&app, &target, "<html><body>second</body></html>");
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    first.send(Ok(html(url, "first"))).expect("first response");
    app.poll_url_source(&egui::Context::default());
    assert_target_body(&app, &target, "<html><body>second</body></html>");
    assert!(app.state.url_tab.pending_url_requests.is_empty());
}

fn assert_target_body(app: &crate::shell::KatanaApp, target: &std::path::Path, expected: &str) {
    assert_eq!(source_body(app, target).as_deref(), Some(expected));
}

#[test]
fn redirected_initial_pdf_failure_preserves_requested_identity() {
    let mut app = app();
    let url = "https://example.test/report.pdf";
    let (sender, pending) = request(url, None);
    app.state.url_tab.pending_url_requests.push_back(pending);
    sender
        .send(Err(HtmlSourceError::HttpStatus {
            url: "https://example.test/login".to_owned(),
            status: HTTP_FORBIDDEN,
            status_text: "Forbidden".to_owned(),
            server: None,
            cloudflare_challenge: false,
        }))
        .expect("redirect failure");
    app.poll_url_source(&egui::Context::default());
    let failure = app
        .tab_previews
        .first()
        .and_then(|preview| preview.pane.document_failure.as_ref())
        .expect("requested PDF failure");
    assert_eq!(failure.document, url);
    assert!(failure.cause.contains("https://example.test/login"));
}

#[test]
fn html_failure_is_bound_to_target_without_claiming_binary_format() {
    let mut app = app();
    let url = "https://example.test/source.html";
    let target = PathBuf::from("existing.html");
    app.state
        .document
        .open_documents
        .push(Document::new(target.clone(), "original"));
    let (sender, pending) = request(url, Some(target.clone()));
    app.state.url_tab.pending_url_requests.push_back(pending);
    app.state.url_tab.input = "https://example.test/unrelated.pdf".to_owned();
    sender
        .send(Err(HtmlSourceError::InvalidUtf8))
        .expect("error response");
    app.poll_url_source(&egui::Context::default());
    assert_html_failure(&app, &target, url);
}

fn assert_html_failure(app: &crate::shell::KatanaApp, target: &std::path::Path, url: &str) {
    let preview = app
        .tab_previews
        .iter()
        .find(|preview| preview.path == target)
        .expect("target preview");
    let failure = preview
        .pane
        .document_failure
        .as_ref()
        .expect("HTML failure");
    assert_eq!(failure.format, None);
    assert_eq!(failure.document, url);
    assert_eq!(
        failure.summary(),
        "source intake could not display this document"
    );
    assert_eq!(app.state.document.open_documents.len(), 1);
    assert_eq!(app.state.document.open_documents[0].buffer, "original");
}

#[test]
fn disconnected_and_expired_requests_keep_their_existing_target() {
    for disconnected in [true, false] {
        assert_existing_target_failure(disconnected);
    }
}

fn assert_existing_target_failure(disconnected: bool) {
    let mut app = app();
    let target = PathBuf::from("existing.pdf");
    app.state
        .document
        .open_documents
        .push(Document::new(target.clone(), "original"));
    let (sender, mut pending) = request("https://example.test/source.pdf", Some(target.clone()));
    if disconnected {
        drop(sender);
    } else {
        pending.deadline = std::time::Instant::now();
    }
    app.state.url_tab.pending_url_requests.push_back(pending);
    app.state.url_tab.input = "https://example.test/unrelated.pptx".to_owned();
    app.poll_url_source(&egui::Context::default());
    assert_target_failure_state(&app, &target);
}

fn assert_target_failure_state(app: &crate::shell::KatanaApp, target: &PathBuf) {
    let preview = app
        .tab_previews
        .iter()
        .find(|preview| preview.path == *target)
        .expect("target preview");
    let failure = preview
        .pane
        .document_failure
        .as_ref()
        .expect("request failure");
    assert_eq!(failure.document, "https://example.test/source.pdf");
    assert_eq!(app.state.document.open_documents.len(), 1);
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);
}
