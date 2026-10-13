use crate::app::workspace::{ExplorerLoadType, WorkspaceOps};
use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, HtmlSource, HtmlSourceError, PendingUrlRequest};
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry, workspace::Workspace};
use katana_platform::{InMemoryCacheService, SettingsService};
use std::{path::PathBuf, sync::Arc, time::Instant};

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

fn make_app() -> KatanaApp {
    let mut state = crate::app_state::AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        SettingsService::default(),
        Arc::new(InMemoryCacheService::default()),
    );
    state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
        katana_platform::workspace::InMemoryWorkspaceRepository::default(),
    ));
    KatanaApp::new(state)
}

fn queue_request(
    app: &mut KatanaApp,
    target_document: Option<PathBuf>,
    source_url: &str,
) -> std::sync::mpsc::Sender<Result<FetchedUrlSource, HtmlSourceError>> {
    let (sender, response_rx) = std::sync::mpsc::channel();
    app.state
        .url_tab
        .pending_url_requests
        .push_back(PendingUrlRequest {
            response_rx,
            target_document,
            source_url: source_url.to_string(),
            deadline: Instant::now() + REQUEST_TIMEOUT,
        });
    app.state.url_tab.is_loading = true;
    sender
}

fn queued_explorer_result(
    app: &mut KatanaApp,
    load_type: ExplorerLoadType,
    result: Result<Workspace, katana_core::workspace::WorkspaceError>,
    path: PathBuf,
) {
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send((load_type, path, result)).unwrap();
    app.explorer_rx = Some(receiver);
    app.state.workspace.is_loading = true;
}

fn stale_html() -> FetchedUrlSource {
    FetchedUrlSource::Html(HtmlSource {
        raw_html: "<html><body>stale HTTP response</body></html>".to_string(),
        source_url: "https://example.test/stale.html".to_string(),
        origin: "https://example.test/stale.html".to_string(),
    })
}

#[test]
fn successful_open_cancels_retargeted_request_but_keeps_unscoped_request() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_root = directory.path().to_path_buf();
    let document_path = workspace_root.join("restored.html");
    let local_html = "<html><body>restored local document</body></html>";
    std::fs::write(&document_path, local_html).unwrap();

    let mut app = make_app();
    let settings = &mut app.state.config.settings.settings_mut().workspace;
    settings.restore_session = true;
    settings.last_workspace = Some(workspace_root.display().to_string());
    settings.open_tabs = vec![document_path.display().to_string()];
    settings.active_tab_idx = Some(0);
    app.state.workspace.data = Some(Workspace::new(&workspace_root, Vec::new()));
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            document_path.clone(),
            "<html><body>previous document instance</body></html>".to_string(),
        ));
    app.state.document.active_doc_idx = Some(0);

    let stale_sender = queue_request(
        &mut app,
        Some(document_path.clone()),
        "https://example.test/stale.html",
    );
    let (none_sender, none_receiver) = std::sync::mpsc::channel();
    drop(none_sender);
    app.state
        .url_tab
        .pending_url_requests
        .push_back(PendingUrlRequest {
            response_rx: none_receiver,
            target_document: None,
            source_url: "https://example.test/unscoped.html".to_string(),
            deadline: Instant::now() + REQUEST_TIMEOUT,
        });

    queued_explorer_result(
        &mut app,
        ExplorerLoadType::Open,
        Ok(Workspace::new(&workspace_root, Vec::new())),
        workspace_root,
    );
    let ctx = egui::Context::default();
    app.poll_explorer_load(&ctx);

    assert!(app.explorer_rx.is_none());
    assert!(!app.state.workspace.is_loading);
    assert_eq!(app.state.document.open_documents.len(), 1);
    assert_eq!(app.state.document.open_documents[0].path, document_path);
    assert_eq!(app.state.document.open_documents[0].buffer, local_html);
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert!(
        app.state.url_tab.pending_url_requests[0]
            .target_document
            .is_none()
    );
    assert!(app.state.url_tab.is_loading);

    assert!(stale_sender.send(Ok(stale_html())).is_err());
    app.poll_url_source(&ctx);

    assert_eq!(app.state.document.open_documents[0].buffer, local_html);
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);
}

#[test]
fn unfinished_open_poll_keeps_targeted_request_for_open_document() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_root = directory.path().to_path_buf();
    let document_path = workspace_root.join("open.html");
    std::fs::write(&document_path, "<html>open</html>").unwrap();

    let mut app = make_app();
    app.state.workspace.data = Some(Workspace::new(&workspace_root, Vec::new()));
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            document_path.clone(),
            "<html>open</html>".to_string(),
        ));
    app.state.document.active_doc_idx = Some(0);
    let sender = queue_request(
        &mut app,
        Some(document_path.clone()),
        "https://example.test/in-progress-target.html",
    );
    let (explorer_sender, explorer_receiver) = std::sync::mpsc::channel();
    app.explorer_rx = Some(explorer_receiver);
    app.state.workspace.is_loading = true;

    let ctx = egui::Context::default();
    app.poll_explorer_load(&ctx);

    assert!(app.explorer_rx.is_some());
    assert!(app.state.workspace.is_loading);
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert_eq!(
        app.state.url_tab.pending_url_requests[0]
            .target_document
            .as_deref(),
        Some(document_path.as_path())
    );
    assert!(app.state.url_tab.is_loading);

    sender
        .send(Err(HtmlSourceError::Network(
            "fixture response".to_string(),
        )))
        .unwrap();
    app.poll_url_source(&ctx);
    assert!(matches!(
        app.state.url_tab.last_error,
        Some(HtmlSourceError::Network(message)) if message == "fixture response"
    ));
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);

    drop(explorer_sender);
}

#[test]
fn successful_refresh_keeps_targeted_request_for_open_document() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_root = directory.path().to_path_buf();
    let document_path = workspace_root.join("open.html");
    std::fs::write(&document_path, "<html>open</html>").unwrap();

    let mut app = make_app();
    app.state.workspace.data = Some(Workspace::new(&workspace_root, Vec::new()));
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            document_path.clone(),
            "<html>open</html>".to_string(),
        ));
    app.state.document.active_doc_idx = Some(0);
    let sender = queue_request(
        &mut app,
        Some(document_path),
        "https://example.test/refresh-target.html",
    );
    queued_explorer_result(
        &mut app,
        ExplorerLoadType::Refresh,
        Ok(Workspace::new(&workspace_root, Vec::new())),
        workspace_root,
    );

    let ctx = egui::Context::default();
    app.poll_explorer_load(&ctx);
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert!(app.state.url_tab.is_loading);
    sender
        .send(Err(HtmlSourceError::Network(
            "fixture response".to_string(),
        )))
        .unwrap();
    app.poll_url_source(&ctx);

    assert!(matches!(
        app.state.url_tab.last_error,
        Some(HtmlSourceError::Network(message)) if message == "fixture response"
    ));
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);
}

#[test]
fn failed_workspace_open_keeps_targeted_request_for_existing_document() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_root = directory.path().to_path_buf();
    let document_path = workspace_root.join("open.html");
    std::fs::write(&document_path, "<html>open</html>").unwrap();

    let mut app = make_app();
    app.state.workspace.data = Some(Workspace::new(&workspace_root, Vec::new()));
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            document_path.clone(),
            "<html>open</html>".to_string(),
        ));
    app.state.document.active_doc_idx = Some(0);
    let sender = queue_request(
        &mut app,
        Some(document_path),
        "https://example.test/error-target.html",
    );
    queued_explorer_result(
        &mut app,
        ExplorerLoadType::Open,
        Err(katana_core::workspace::WorkspaceError::NoWorkspace),
        workspace_root,
    );

    let ctx = egui::Context::default();
    app.poll_explorer_load(&ctx);
    assert_eq!(app.state.document.open_documents.len(), 1);
    assert_eq!(app.state.url_tab.pending_url_requests.len(), 1);
    assert!(app.state.url_tab.is_loading);
    sender
        .send(Err(HtmlSourceError::Network(
            "fixture response".to_string(),
        )))
        .unwrap();
    app.poll_url_source(&ctx);

    assert!(matches!(
        app.state.url_tab.last_error,
        Some(HtmlSourceError::Network(message)) if message == "fixture response"
    ));
    assert!(app.state.url_tab.pending_url_requests.is_empty());
    assert!(!app.state.url_tab.is_loading);
}
