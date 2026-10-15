use crate::app_state::AppState;
use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, HtmlSource, HtmlSourceError, PendingUrlRequest};
use katana_core::document::Document;
use std::sync::{Arc, mpsc};
use std::time::Instant;

mod edge_cases;
mod format_collision;
mod mixed_targets;
mod reload_tests;
mod selection_enqueue_tests;
mod selection_failure_tests;
mod selection_tests;

fn app() -> KatanaApp {
    KatanaApp::new(AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    ))
}

fn html(url: &str, body: &str) -> FetchedUrlSource {
    FetchedUrlSource::Html(HtmlSource {
        raw_html: format!("<html><body>{body}</body></html>"),
        source_url: url.to_owned(),
        origin: url.to_owned(),
    })
}

fn request(
    source_url: &str,
    target_document: Option<std::path::PathBuf>,
) -> (
    mpsc::Sender<Result<FetchedUrlSource, HtmlSourceError>>,
    PendingUrlRequest,
) {
    let (sender, response_rx) = mpsc::channel();
    (
        sender,
        PendingUrlRequest {
            response_rx,
            target_document,
            source_url: source_url.to_owned(),
            order: None,
            selection_owner: None,
            deadline: Instant::now() + super::URL_SOURCE_TIMEOUT,
        },
    )
}

fn source_body(app: &KatanaApp, path: &std::path::Path) -> Option<String> {
    app.state
        .url_tab
        .source_for_document(path)
        .map(|source| source.raw_html.clone())
}

fn open_html_targets(app: &mut KatanaApp, a_path: &std::path::Path, b_path: &std::path::Path) {
    app.state
        .document
        .open_documents
        .push(Document::new(a_path.to_path_buf(), "A"));
    app.state
        .document
        .open_documents
        .push(Document::new(b_path.to_path_buf(), "B"));
}

fn send_html(
    sender: &mpsc::Sender<Result<FetchedUrlSource, HtmlSourceError>>,
    url: &str,
    body: &str,
) {
    sender.send(Ok(html(url, body))).unwrap();
}

fn assert_body(app: &KatanaApp, path: &std::path::Path, body: &str) {
    assert_eq!(source_body(app, path).as_deref(), Some(body));
}

#[test]
fn ready_later_request_updates_while_earlier_request_keeps_loading() {
    let mut app = app();
    let a_path = super::document::remote_document_path("https://example.test/a.html", "html");
    let b_path = super::document::remote_document_path("https://example.test/b.html", "html");
    open_html_targets(&mut app, &a_path, &b_path);
    let (a_sender, a_request) = request("https://example.test/a.html", Some(a_path));
    let (b_sender, b_request) = request("https://example.test/b.html", Some(b_path.clone()));
    app.state.url_tab.pending_url_requests.push_back(a_request);
    app.state.url_tab.pending_url_requests.push_back(b_request);
    app.state.url_tab.is_loading = true;
    send_html(&b_sender, "https://example.test/b.html", "B");

    app.poll_url_source(&egui::Context::default());

    assert_body(&app, &b_path, "<html><body>B</body></html>");
    assert!(app.state.url_tab.is_loading);
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    drop(a_sender);
}

#[test]
fn same_target_ready_update_waits_for_older_request_then_wins() {
    let mut app = app();
    let target = super::document::remote_document_path("https://example.test/a.html", "html");
    let b_path = super::document::remote_document_path("https://example.test/b.html", "html");
    open_html_targets(&mut app, &target, &b_path);
    let (a1_sender, a1_request) = request("https://example.test/a.html", Some(target.clone()));
    let (a2_sender, a2_request) = request("https://example.test/a.html", Some(target.clone()));
    let (b_sender, b_request) = request("https://example.test/b.html", Some(b_path.clone()));
    app.state.url_tab.pending_url_requests.push_back(a1_request);
    app.state.url_tab.pending_url_requests.push_back(a2_request);
    app.state.url_tab.pending_url_requests.push_back(b_request);
    send_html(&a2_sender, "https://example.test/a.html", "A2");
    send_html(&b_sender, "https://example.test/b.html", "B");

    app.poll_url_source(&egui::Context::default());
    assert_body(&app, &b_path, "<html><body>B</body></html>");
    assert_eq!(source_body(&app, &target), None);

    send_html(&a1_sender, "https://example.test/a.html", "A1");
    app.poll_url_source(&egui::Context::default());
    assert_body(&app, &target, "<html><body>A2</body></html>");
}

#[test]
fn failure_request_uses_immutable_source_identity_for_all_intake_errors() {
    for error in [
        HtmlSourceError::Network("offline".to_owned()),
        HtmlSourceError::InvalidUtf8,
        HtmlSourceError::BodyTooLarge {
            limit: 1,
            actual: 2,
        },
    ] {
        assert_failure(error);
    }
}

fn assert_failure(error: HtmlSourceError) {
    let (mut app, a_path, b_path) = failure_fixture(error);
    let source_url = "https://example.test/a.pdf";

    app.poll_url_source(&egui::Context::default());

    let a_preview = app
        .tab_previews
        .iter()
        .find(|preview| preview.path == a_path);
    let failure = a_preview
        .and_then(|preview| preview.pane.document_failure_for_test())
        .expect("A failure preview");
    assert!(failure.contains(source_url));
    assert!(
        !app.tab_previews
            .iter()
            .any(|preview| preview.path == b_path)
    );
    assert_eq!(app.state.document.open_documents.len(), 2);
    let preserved = app
        .state
        .document
        .open_documents
        .iter()
        .find(|document| document.path == b_path)
        .expect("existing B document");
    assert_eq!(preserved.buffer, "preserved");
}

fn failure_fixture(error: HtmlSourceError) -> (KatanaApp, std::path::PathBuf, std::path::PathBuf) {
    let mut app = app();
    let source_url = "https://example.test/a.pdf";
    let input_url = "https://example.test/b.pdf";
    let (_, a_path) = super::document::failed_document_identity(source_url).unwrap();
    let (_, b_path) = super::document::failed_document_identity(input_url).unwrap();
    app.state
        .document
        .open_documents
        .push(Document::new(a_path.clone(), "existing A"));
    app.state
        .document
        .open_documents
        .push(Document::new(b_path.clone(), "preserved"));
    let (sender, request) = request(source_url, Some(a_path.clone()));
    app.state.url_tab.pending_url_requests.push_back(request);
    app.state.url_tab.input = input_url.to_owned();
    sender.send(Err(error)).unwrap();
    (app, a_path, b_path)
}
