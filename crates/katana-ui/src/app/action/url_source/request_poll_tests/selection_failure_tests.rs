use super::{app, html, open_html_targets, request, source_body};
use crate::state::{BinaryUrlSource, FetchedUrlSource, HtmlSourceError};
use katana_core::document_source::BinaryDocumentFormat;
use std::path::{Path, PathBuf};

const A_URL: &str = "https://example.test/a.html";
const B_URL: &str = "https://example.test/b.html";

#[test]
fn older_target_failure_keeps_newer_selection_and_input() {
    let mut app = app();
    let a_path = super::super::document::remote_document_path(A_URL, "html");
    let b_path = super::super::document::remote_document_path(B_URL, "html");
    open_html_targets(&mut app, &a_path, &b_path);
    let (a_sender, a_request) = request(A_URL, Some(a_path.clone()));
    let (b_sender, b_request) = request(B_URL, Some(b_path.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([a_request, b_request]);
    b_sender.send(Ok(html(B_URL, "B"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    a_sender.send(Err(HtmlSourceError::InvalidUtf8)).unwrap();
    app.poll_url_source(&egui::Context::default());

    assert_selected_b(&app, &b_path);
    assert!(app.state.url_tab.last_error.is_none());
    assert!(app.state.layout.status_message.is_none());
    assert_failure_for(&app, &a_path, A_URL);
}

#[test]
fn older_html_to_pdf_migration_keeps_newer_document_selected() {
    let mut app = app();
    let a_path = super::super::document::remote_document_path(A_URL, "html");
    let b_path = super::super::document::remote_document_path(B_URL, "html");
    open_html_targets(&mut app, &a_path, &b_path);
    let (a_sender, a_request) = request(A_URL, Some(a_path.clone()));
    app.state.url_tab.pending_url_requests.push_back(a_request);
    let (b_sender, b_request) = request(B_URL, Some(b_path.clone()));
    app.state.url_tab.pending_url_requests.push_back(b_request);
    b_sender.send(Ok(html(B_URL, "B"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    a_sender.send(Ok(pdf(A_URL))).unwrap();
    app.poll_url_source(&egui::Context::default());

    let pdf_path = super::super::document::remote_document_path(A_URL, "pdf");
    assert_selected_b(&app, &b_path);
    assert_migrated(&app, &a_path, &pdf_path);
}

fn assert_selected_b(app: &crate::shell::KatanaApp, path: &Path) {
    assert_eq!(app.state.active_path(), Some(path.to_path_buf()));
    assert_eq!(app.state.url_tab.input, B_URL);
    assert_eq!(
        source_body(app, path).as_deref(),
        Some("<html><body>B</body></html>")
    );
}

fn assert_migrated(app: &crate::shell::KatanaApp, previous: &Path, next: &Path) {
    assert!(
        app.state
            .document
            .open_documents
            .iter()
            .any(|doc| doc.path == *next)
    );
    assert!(
        !app.state
            .document
            .open_documents
            .iter()
            .any(|doc| doc.path == *previous)
    );
}

#[test]
fn late_migration_follows_a_manually_reselected_target() {
    let mut app = app();
    let a_path = super::super::document::remote_document_path(A_URL, "html");
    let b_path = super::super::document::remote_document_path(B_URL, "html");
    open_html_targets(&mut app, &a_path, &b_path);
    let (a_sender, a_request) = request(A_URL, Some(a_path.clone()));
    let (b_sender, b_request) = request(B_URL, Some(b_path));
    app.state
        .url_tab
        .pending_url_requests
        .extend([a_request, b_request]);
    b_sender.send(Ok(html(B_URL, "B"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    app.state.document.active_doc_idx = Some(0);
    a_sender.send(Ok(pdf(A_URL))).unwrap();
    app.poll_url_source(&egui::Context::default());
    let pdf_path = super::super::document::remote_document_path(A_URL, "pdf");
    assert_migrated(&app, &a_path, &pdf_path);
    assert_eq!(app.state.active_path(), Some(pdf_path));
}

#[test]
fn older_success_does_not_clear_newer_targetless_failure() {
    let mut app = app();
    let a_path = PathBuf::from("a.html");
    open_html_targets(&mut app, &a_path, &PathBuf::from("other.md"));
    let (a_sender, a_request) = request(A_URL, Some(a_path));
    let (b_sender, b_request) = request(B_URL, None);
    app.state
        .url_tab
        .pending_url_requests
        .extend([a_request, b_request]);
    app.state.url_tab.input = B_URL.to_owned();
    b_sender.send(Err(HtmlSourceError::InvalidUtf8)).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert!(app.state.url_tab.last_error.is_some());
    a_sender.send(Ok(html(A_URL, "A"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert!(app.state.url_tab.last_error.is_some());
    assert!(app.state.layout.status_message.is_some());
    assert_eq!(app.state.url_tab.input, B_URL);
}

fn assert_failure_for(app: &crate::shell::KatanaApp, path: &Path, url: &str) {
    let failure = app
        .tab_previews
        .iter()
        .find(|preview| preview.path == *path)
        .and_then(|preview| preview.pane.document_failure.as_ref())
        .expect("A structured failure");
    assert_eq!(failure.document, url);
    assert_eq!(failure.format, None);
}

fn pdf(url: &str) -> FetchedUrlSource {
    FetchedUrlSource::Document(BinaryUrlSource {
        bytes: b"%PDF-1.7".to_vec(),
        source_url: url.to_owned(),
        mime: BinaryDocumentFormat::Pdf.mime().to_owned(),
        format: BinaryDocumentFormat::Pdf,
    })
}
