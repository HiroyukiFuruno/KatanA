use super::{app, html, request};
use crate::app::action::FileOpenOps;
use crate::app::document_contract::DocumentOps;
use crate::app_state::StatusType;
use katana_core::document::Document;
use std::path::PathBuf;
use std::sync::Arc;

const STALE_URL: &str = "https://example.test/stale.html";
const LOCAL_INPUT: &str = "local file input";

#[test]
fn file_open_selection_preserves_local_state_from_stale_response()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let workspace_root = directory.path().to_path_buf();
    let local_path = workspace_root.join("local.md");
    std::fs::write(&local_path, "local markdown")?;
    let mut app = app();
    app.state.workspace.data = Some(katana_core::workspace::Workspace::new(
        &workspace_root,
        Vec::new(),
    ));
    let (sender, mut pending) = request(STALE_URL, None);
    pending.selection_owner = Some(app.state.url_tab.selection_owner.clone());
    app.state.url_tab.pending_url_requests.push_back(pending);
    FileOpenOps::open_in_current_workspace(&mut app, local_path.clone());
    let status = Some(("local file status".to_owned(), StatusType::Info));
    app.state.url_tab.input = LOCAL_INPUT.to_owned();
    app.state.layout.status_message = status.clone();
    sender.send(Ok(html(STALE_URL, "stale HTTP response")))?;
    app.poll_url_source(&egui::Context::default());

    assert_local_state(&app, local_path, status);
    Ok(())
}

fn assert_local_state(
    app: &crate::shell::KatanaApp,
    local_path: PathBuf,
    status: Option<(String, StatusType)>,
) {
    assert_eq!(app.state.active_path(), Some(local_path));
    assert_eq!(app.state.url_tab.input, LOCAL_INPUT);
    assert_eq!(app.state.layout.status_message, status);
    assert_eq!(
        app.state.active_document().unwrap().buffer,
        "local markdown"
    );
    assert!(app.state.url_tab.tabs.iter().any(|tab| {
        tab.source.source_url == STALE_URL
            && tab.source.raw_html == "<html><body>stale HTTP response</body></html>"
    }));
}

#[test]
fn same_path_activation_keeps_selection_owner() {
    let (mut app, path) = local_document_fixture();
    let before = app.state.url_tab.selection_owner.clone();
    app.handle_select_document(path, true);
    assert!(Arc::ptr_eq(&before, &app.state.url_tab.selection_owner));
}

#[test]
fn background_open_keeps_selection_owner() {
    let (mut app, path) = local_document_fixture();
    let before = app.state.url_tab.selection_owner.clone();
    app.handle_select_document(PathBuf::from("other.md"), false);
    assert!(Arc::ptr_eq(&before, &app.state.url_tab.selection_owner));
    assert_eq!(app.state.active_path(), Some(path));
}

fn local_document_fixture() -> (crate::shell::KatanaApp, PathBuf) {
    let path = PathBuf::from("same.md");
    let mut app = app();
    app.state
        .document
        .open_documents
        .push(Document::new(path.clone(), "same document"));
    app.state.document.active_doc_idx = Some(0);
    (app, path)
}
