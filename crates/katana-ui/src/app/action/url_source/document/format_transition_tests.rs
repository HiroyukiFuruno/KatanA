use crate::app_state::AppState;
use crate::shell::KatanaApp;
use crate::state::{BinaryUrlSource, FetchedUrlSource};
use crate::views::panels::preview::{PreviewMenu, PreviewMenuAvailability};
use katana_core::document::Document;
use katana_core::document_source::BinaryDocumentFormat;
use std::path::PathBuf;
use std::sync::Arc;

fn app() -> KatanaApp {
    KatanaApp::new(AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    ))
}

fn source(format: BinaryDocumentFormat) -> FetchedUrlSource {
    FetchedUrlSource::Document(BinaryUrlSource {
        bytes: if format == BinaryDocumentFormat::Pdf {
            b"%PDF-1.7".to_vec()
        } else {
            vec![0x50, 0x4b, 0x05, 0x06]
        },
        source_url: "https://example.test/download".to_owned(),
        mime: format.mime().to_owned(),
        format,
    })
}

#[test]
fn binary_navigation_retags_html_target_without_reordering_or_unpinning() {
    let mut app = app();
    let previous = PathBuf::from("Katana://URL/source.html");
    let mut document = Document::new(previous.clone(), String::new());
    document.is_pinned = true;
    app.state.document.open_documents.push(document);
    app.state
        .document
        .open_documents
        .push(Document::new(PathBuf::from("other.md"), "other".to_owned()));
    app.state.initialize_tab_split_state(previous.clone());
    let split = app.state.document.tab_split_states[0].state;
    app.state
        .document
        .tab_groups
        .push(crate::state::document::TabGroup {
            id: "group".to_owned(),
            name: "group".to_owned(),
            color_hex: String::new(),
            collapsed: false,
            members: vec![previous.to_string_lossy().into_owned()],
        });
    app.apply_fetched_url_source(source(BinaryDocumentFormat::Pdf), Some(previous.clone()));
    let active = app.state.active_path().expect("active binary document");
    assert_eq!(
        active.extension().and_then(|value| value.to_str()),
        Some("pdf")
    );
    assert_eq!(app.state.document.open_documents.len(), 2);
    assert_eq!(app.state.document.active_doc_idx, Some(0));
    assert!(app.state.document.open_documents[0].is_pinned);
    assert_eq!(app.state.document.tab_split_states[0].state, split);
    assert_eq!(
        app.state.document.tab_groups[0].members,
        vec![active.to_string_lossy().into_owned()]
    );
    assert!(PreviewMenuAvailability::for_path(
        Some(&active),
        PreviewMenu::Tools
    ));
    assert!(
        app.state
            .document
            .tab_split_states
            .iter()
            .all(|tab| tab.path != previous)
    );
    assert_eq!(
        app.state.url_tab.document_source_url_for_document(&active),
        Some("https://example.test/download")
    );
}

#[test]
fn binary_navigation_pptx_identity_enables_slideshow_format_classification() {
    let mut app = app();
    let previous = PathBuf::from("Katana://URL/source.html");
    app.state
        .document
        .open_documents
        .push(Document::new(previous.clone(), String::new()));
    app.apply_fetched_url_source(source(BinaryDocumentFormat::Pptx), Some(previous));
    assert!(PreviewMenuAvailability::for_path(
        app.state.active_path().as_deref(),
        PreviewMenu::Slideshow
    ));
    assert_eq!(
        BinaryDocumentFormat::from_path(&app.state.active_path().expect("active path")),
        Some(BinaryDocumentFormat::Pptx)
    );
}

#[test]
fn binary_refresh_keeps_same_format_path_and_reuses_existing_target() {
    let mut app = app();
    let existing = PathBuf::from("Katana://URL/existing.pdf");
    app.state
        .document
        .open_documents
        .push(Document::new(existing.clone(), String::new()));
    app.apply_fetched_url_source(source(BinaryDocumentFormat::Pdf), Some(existing.clone()));
    assert_eq!(app.state.active_path(), Some(existing));
    assert_eq!(app.state.document.open_documents.len(), 1);
}

#[test]
fn binary_navigation_collision_keeps_both_documents_and_target_state() {
    let mut app = app();
    let previous = PathBuf::from("Katana://URL/source.html");
    let target = super::remote_document_path("https://example.test/download", "pdf");
    app.state
        .document
        .open_documents
        .push(Document::new(previous.clone(), "source"));
    let mut document = Document::new(target.clone(), "preserved");
    document.is_pinned = true;
    app.state.document.open_documents.push(document);
    app.apply_fetched_url_source(source(BinaryDocumentFormat::Pdf), Some(previous.clone()));
    assert_eq!(app.state.document.open_documents.len(), 2);
    assert_eq!(app.state.document.open_documents[0].path, previous);
    assert_eq!(app.state.document.open_documents[0].buffer, "source");
    assert_eq!(app.state.active_path(), Some(target));
    assert_eq!(app.state.document.open_documents[1].buffer, "preserved");
    assert!(app.state.document.open_documents[1].is_pinned);
}

#[test]
fn binary_navigation_preserves_unsaved_html_in_its_original_tab() {
    let mut app = app();
    let previous = PathBuf::from("local.html");
    let mut document = Document::new(previous.clone(), "unsaved html".to_owned());
    document.is_dirty = true;
    app.state.document.open_documents.push(document);
    app.apply_fetched_url_source(source(BinaryDocumentFormat::Pdf), Some(previous.clone()));
    assert_eq!(app.state.document.open_documents.len(), 2);
    assert_eq!(app.state.document.open_documents[0].path, previous);
    assert_eq!(app.state.document.open_documents[0].buffer, "unsaved html");
    assert!(app.state.document.open_documents[0].is_dirty);
    assert_eq!(
        BinaryDocumentFormat::from_path(&app.state.active_path().expect("active path")),
        Some(BinaryDocumentFormat::Pdf)
    );
}
