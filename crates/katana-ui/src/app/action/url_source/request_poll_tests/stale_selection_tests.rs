use super::{app, request};
use crate::shell::KatanaApp;
use crate::state::{BinaryUrlSource, FetchedUrlSource, HtmlSourceError};
use katana_core::document::Document;
use katana_core::document_source::BinaryDocumentFormat;
use std::path::PathBuf;
use std::sync::Arc;

const C_URL: &str = "https://example.test/source.docx";
const D_URL: &str = "https://example.test/destination.pdf";
const ALIAS_URL: &str = "https://example.test/alias.pdf";
const HTTP_INTERNAL_SERVER_ERROR: u16 = 500;

#[test]
fn newer_auto_migration_then_old_user_completion_keeps_new_payload() {
    let (mut app, c_path, d_path) = seeded_app();
    let owner = app.state.url_tab.selection_owner.clone();
    let (user_tx, mut user) = request(ALIAS_URL, None);
    user.selection_owner = Some(owner);
    let (auto_tx, mut auto) = request(C_URL, Some(c_path.clone()));
    auto.selection_owner = Some(Arc::new(()));
    app.state.url_tab.pending_url_requests.extend([user, auto]);
    auto_tx.send(Ok(pdf("new"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    let generation = destination_generation(&app, &d_path);
    assert_eq!(app.state.active_path(), Some(c_path.clone()));
    user_tx.send(Ok(pdf("old"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(destination_generation(&app, &d_path), generation);
    assert_selected_destination(&app, &d_path);
}

#[test]
fn changed_manual_owner_does_not_select_destination_from_old_user_completion() {
    let (mut app, c_path, d_path) = seeded_app();
    let (sender, mut user) = request(ALIAS_URL, None);
    user.selection_owner = Some(app.state.url_tab.selection_owner.clone());
    let (auto_tx, mut auto) = request(C_URL, Some(c_path.clone()));
    auto.selection_owner = Some(Arc::new(()));
    app.state.url_tab.pending_url_requests.extend([user, auto]);
    auto_tx.send(Ok(pdf("new"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    let generation = destination_generation(&app, &d_path);
    app.state.url_tab.selection_owner = Arc::new(());
    sender.send(Ok(pdf("old"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.active_path(), Some(c_path));
    assert_eq!(destination_generation(&app, &d_path), generation);
    assert_eq!(app.state.url_tab.input, ALIAS_URL);
}

#[test]
fn stale_user_failure_keeps_new_destination_without_reapplying_error() {
    let (mut app, c_path, d_path) = seeded_app();
    let owner = app.state.url_tab.selection_owner.clone();
    let (user_tx, mut user) = request(D_URL, None);
    user.selection_owner = Some(owner);
    let (auto_tx, mut auto) = request(C_URL, Some(c_path));
    auto.selection_owner = Some(Arc::new(()));
    app.state.url_tab.pending_url_requests.extend([user, auto]);
    auto_tx.send(Ok(pdf("new"))).unwrap();
    app.poll_url_source(&egui::Context::default());
    let generation = destination_generation(&app, &d_path);
    user_tx.send(Err(http_failure())).unwrap();
    app.poll_url_source(&egui::Context::default());
    assert_eq!(destination_generation(&app, &d_path), generation);
    assert_selected_destination(&app, &d_path);
    assert!(app.state.url_tab.last_error.is_none());
    assert!(app.state.layout.status_message.is_none());
}

fn seeded_app() -> (KatanaApp, PathBuf, PathBuf) {
    let mut app = app();
    let c_path = super::super::document::remote_document_path(C_URL, "docx");
    let d_path = super::super::document::remote_document_path(D_URL, "pdf");
    app.state
        .document
        .open_documents
        .push(Document::new(c_path.clone(), "C"));
    app.state
        .document
        .open_documents
        .push(Document::new(d_path.clone(), "D"));
    app.state.document.active_doc_idx = Some(0);
    app.state
        .url_tab
        .open_document_source(D_URL.to_owned(), d_path.clone());
    app.state.url_tab.input = ALIAS_URL.to_owned();
    (app, c_path, d_path)
}

fn pdf(body: &str) -> FetchedUrlSource {
    FetchedUrlSource::Document(BinaryUrlSource {
        bytes: format!("%PDF-1.7\n{body}").into_bytes(),
        source_url: D_URL.to_owned(),
        mime: BinaryDocumentFormat::Pdf.mime().to_owned(),
        format: BinaryDocumentFormat::Pdf,
    })
}

fn http_failure() -> HtmlSourceError {
    HtmlSourceError::HttpStatus {
        status: HTTP_INTERNAL_SERVER_ERROR,
        status_text: "old response".to_owned(),
        url: D_URL.to_owned(),
        server: None,
        cloudflare_challenge: false,
    }
}

fn destination_generation(app: &KatanaApp, path: &std::path::Path) -> u64 {
    app.tab_previews
        .iter()
        .find(|preview| preview.path == path)
        .expect("destination preview")
        .pane
        .session_generation
}

fn assert_selected_destination(app: &KatanaApp, path: &std::path::Path) {
    assert_eq!(app.state.active_path(), Some(path.to_path_buf()));
    assert_eq!(app.state.url_tab.input, D_URL);
    assert_eq!(
        app.state.url_tab.document_source_url_for_document(path),
        Some(D_URL)
    );
}
