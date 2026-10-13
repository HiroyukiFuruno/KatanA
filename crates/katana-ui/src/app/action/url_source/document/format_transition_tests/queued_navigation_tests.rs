use super::{app, source, source_at};
use crate::state::FetchedUrlSource;
use katana_core::document::Document;
use katana_core::document_source::BinaryDocumentFormat;
use std::path::PathBuf;

fn html_source(source_url: &str) -> FetchedUrlSource {
    FetchedUrlSource::Html(crate::state::HtmlSource {
        raw_html: "<html><body>second response HTML</body></html>".to_owned(),
        source_url: source_url.to_owned(),
        origin: source_url.to_owned(),
    })
}

#[test]
fn queued_binary_responses_follow_clean_target_migration() {
    let ctx = egui::Context::default();
    for (first_format, format) in [
        (BinaryDocumentFormat::Pdf, BinaryDocumentFormat::Pdf),
        (BinaryDocumentFormat::Docx, BinaryDocumentFormat::Docx),
        (BinaryDocumentFormat::Pdf, BinaryDocumentFormat::Docx),
        (BinaryDocumentFormat::Docx, BinaryDocumentFormat::Pdf),
    ] {
        let mut app = app();
        let html_path = PathBuf::from("Katana://URL/source.html");
        app.state
            .document
            .open_documents
            .push(Document::new(html_path.clone(), "html"));
        let first_url = format!("https://example.test/first.{}", first_format.extension());
        let second_url = format!("https://example.test/second.{}", format.extension());
        let (first_sender, first_receiver) = std::sync::mpsc::channel();
        let (second_sender, second_receiver) = std::sync::mpsc::channel();
        app.state
            .url_tab
            .pending_url_requests
            .push_back(crate::state::PendingUrlRequest {
                response_rx: first_receiver,
                target_document: Some(html_path.clone()),
                source_url: first_url.to_owned(),
                deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
            });
        app.state
            .url_tab
            .pending_url_requests
            .push_back(crate::state::PendingUrlRequest {
                response_rx: second_receiver,
                target_document: Some(html_path.clone()),
                source_url: second_url.to_owned(),
                deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
            });
        first_sender
            .send(Ok(source_at(first_format, &first_url)))
            .expect("first response");
        second_sender
            .send(Ok(source_at(format, &second_url)))
            .expect("second response");
        app.state.url_tab.is_loading = true;

        app.poll_url_source(&ctx);

        let migrated = super::super::remote_document_path(
            if first_format == format {
                &first_url
            } else {
                &second_url
            },
            format.extension(),
        );
        assert_eq!(app.state.document.open_documents.len(), 1);
        assert_eq!(app.state.document.open_documents[0].path, migrated);
        assert_eq!(
            app.state
                .url_tab
                .document_source_url_for_document(&migrated),
            Some(second_url.as_str())
        );
        assert!(app.state.url_tab.pending_url_requests.is_empty());
    }
}

#[test]
fn queued_html_response_migrates_binary_target_back_to_html_identity() {
    let ctx = egui::Context::default();
    let mut app = app();
    let html_path = PathBuf::from("Katana://URL/source.html");
    let mut original = Document::new(html_path.clone(), "original html");
    original.is_pinned = true;
    app.state.document.open_documents.push(original);
    let first_url = "https://example.test/first.pdf";
    let second_url = "https://example.test/second.html";
    let (first_sender, first_receiver) = std::sync::mpsc::channel();
    let (second_sender, second_receiver) = std::sync::mpsc::channel();
    app.state
        .url_tab
        .pending_url_requests
        .push_back(crate::state::PendingUrlRequest {
            response_rx: first_receiver,
            target_document: Some(html_path.clone()),
            source_url: first_url.to_owned(),
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
        });
    app.state
        .url_tab
        .pending_url_requests
        .push_back(crate::state::PendingUrlRequest {
            response_rx: second_receiver,
            target_document: Some(html_path),
            source_url: second_url.to_owned(),
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
        });
    first_sender
        .send(Ok(source_at(BinaryDocumentFormat::Pdf, first_url)))
        .expect("first response");
    second_sender
        .send(Ok(html_source(second_url)))
        .expect("second response");

    app.poll_url_source(&ctx);

    let html_target = super::super::remote_document_path(second_url, "html");
    assert_eq!(app.state.document.open_documents.len(), 1);
    assert_eq!(app.state.active_path(), Some(html_target.clone()));
    assert_eq!(
        app.state.document.open_documents[0].buffer,
        "<html><body>second response HTML</body></html>"
    );
    assert!(app.state.document.open_documents[0].is_pinned);
    assert_eq!(
        app.state
            .url_tab
            .source_for_document(&html_target)
            .map(|html| html.source_url.as_str()),
        Some(second_url)
    );
    assert_eq!(
        BinaryDocumentFormat::from_path(&html_target),
        None,
        "HTML response must not retain the prior PDF identity"
    );
    assert!(app.state.url_tab.pending_url_requests.is_empty());
}

#[test]
fn closing_migrated_target_cancels_retargeted_pending_request() {
    use crate::app::action::ActionOps;
    use crate::app_state::AppAction;

    let ctx = egui::Context::default();
    let mut app = app();
    let html_path = PathBuf::from("Katana://URL/source.html");
    app.state
        .document
        .open_documents
        .push(Document::new(html_path.clone(), "html"));
    let (first_sender, first_receiver) = std::sync::mpsc::channel();
    let (_second_sender, second_receiver) = std::sync::mpsc::channel();
    app.state
        .url_tab
        .pending_url_requests
        .push_back(crate::state::PendingUrlRequest {
            response_rx: first_receiver,
            target_document: Some(html_path.clone()),
            source_url: "https://example.test/first.pdf".to_owned(),
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
        });
    app.state
        .url_tab
        .pending_url_requests
        .push_back(crate::state::PendingUrlRequest {
            response_rx: second_receiver,
            target_document: Some(html_path),
            source_url: "https://example.test/second.pdf".to_owned(),
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
        });
    first_sender
        .send(Ok(source_at(
            BinaryDocumentFormat::Pdf,
            "https://example.test/first.pdf",
        )))
        .expect("first response");

    app.poll_url_source(&ctx);

    let migrated = super::super::remote_document_path("https://example.test/first.pdf", "pdf");
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert_eq!(
        app.state.url_tab.pending_url_requests[0].target_document,
        Some(migrated)
    );
    app.process_action(&ctx, AppAction::ForceCloseDocument(0));
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);
}

#[test]
fn preserved_dirty_or_collision_target_does_not_retarget_pending_request() {
    for collision in [false, true] {
        let mut app = app();
        let html_path = PathBuf::from("Katana://URL/source.html");
        let target = super::super::remote_document_path("https://example.test/download", "pdf");
        let mut original = Document::new(html_path.clone(), "preserved html");
        original.is_dirty = !collision;
        app.state.document.open_documents.push(original);
        if collision {
            app.state
                .document
                .open_documents
                .push(Document::new(target, "preserved collision"));
        }
        let (_sender, receiver) = std::sync::mpsc::channel();
        app.state
            .url_tab
            .pending_url_requests
            .push_back(crate::state::PendingUrlRequest {
                response_rx: receiver,
                target_document: Some(html_path.clone()),
                source_url: "https://example.test/pending.pdf".to_owned(),
                deadline: std::time::Instant::now() + std::time::Duration::from_secs(30),
            });

        app.apply_fetched_url_source(source(BinaryDocumentFormat::Pdf), Some(html_path.clone()));

        assert_eq!(
            app.state.url_tab.pending_url_requests[0].target_document,
            Some(html_path.clone())
        );
        assert!(
            app.state.document.open_documents.iter().any(|document| {
                document.path == html_path && document.buffer == "preserved html"
            })
        );
    }
}
