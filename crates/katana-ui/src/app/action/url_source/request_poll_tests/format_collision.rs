use super::{app, open_html_targets, request};
use crate::state::{BinaryUrlSource, FetchedUrlSource};
use katana_core::document_source::BinaryDocumentFormat;

const URL: &str = "https://example.test/report.pdf";

#[test]
fn navigation_and_existing_destination_refresh_keep_same_source_order() {
    let mut app = app();
    let previous =
        super::super::document::remote_document_path("https://example.test/a.html", "html");
    let target = super::super::document::remote_document_path(URL, "pdf");
    open_html_targets(&mut app, &previous, &target);
    let (first, pending) = request(URL, Some(previous));
    let (second, ready) = request(URL, Some(target.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    second.send(Ok(pdf("new"))).expect("newer refresh");
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 2);
    assert_eq!(
        app.state.url_tab.document_source_url_for_document(&target),
        None
    );
    first.send(Ok(pdf("old"))).expect("older navigation");
    app.poll_url_source(&egui::Context::default());
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert_eq!(app.state.document.open_documents.len(), 2);
    assert_eq!(
        app.state.url_tab.document_source_url_for_document(&target),
        Some(URL)
    );
}

fn pdf(body: &str) -> FetchedUrlSource {
    FetchedUrlSource::Document(BinaryUrlSource {
        bytes: format!("%PDF-1.7\n{body}").into_bytes(),
        source_url: URL.to_owned(),
        mime: BinaryDocumentFormat::Pdf.mime().to_owned(),
        format: BinaryDocumentFormat::Pdf,
    })
}

#[test]
fn redirected_older_navigation_does_not_replace_newer_destination() {
    let mut app = app();
    let previous =
        super::super::document::remote_document_path("https://example.test/a.html", "html");
    let target = super::super::document::remote_document_path(URL, "pdf");
    open_html_targets(&mut app, &previous, &target);
    let (first, pending) = request("https://example.test/alias.pdf", Some(previous));
    let (second, ready) = request(URL, Some(target.clone()));
    app.state
        .url_tab
        .pending_url_requests
        .extend([pending, ready]);
    second.send(Ok(pdf("new"))).expect("newer refresh");
    app.poll_url_source(&egui::Context::default());
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    let generation = destination_generation(&app, &target);
    first
        .send(Ok(pdf("old")))
        .expect("redirected older navigation");
    app.poll_url_source(&egui::Context::default());
    assert_eq!(destination_generation(&app, &target), generation);
    assert!(app.state.url_tab.pending_url_requests.is_empty());
}

fn destination_generation(app: &crate::shell::KatanaApp, target: &std::path::Path) -> u64 {
    app.tab_previews
        .iter()
        .find(|preview| preview.path == target)
        .expect("destination preview")
        .pane
        .session_generation
}

#[test]
fn new_destination_refresh_after_empty_queue_is_not_discarded_as_stale() {
    let mut app = app();
    let target = super::super::document::remote_document_path(URL, "pdf");
    open_html_targets(&mut app, &target, &std::path::PathBuf::from("other.html"));
    for body in ["first", "next"] {
        let before = app
            .tab_previews
            .iter()
            .find(|preview| preview.path == target)
            .map_or(0, |preview| preview.pane.session_generation);
        let (sender, pending) = request(URL, Some(target.clone()));
        app.state.url_tab.pending_url_requests.push_back(pending);
        sender.send(Ok(pdf(body))).expect("refresh");
        app.poll_url_source(&egui::Context::default());
        assert!(destination_generation(&app, &target) > before);
        assert!(app.state.url_tab.pending_url_requests.is_empty());
    }
}
