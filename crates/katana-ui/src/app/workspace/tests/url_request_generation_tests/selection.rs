use super::{make_app, queue_request, queued_explorer_result, stale_html};
use crate::app::workspace::{ExplorerLoadType, WorkspaceOps};
use crate::app_state::StatusType;
use std::path::Path;

const LOCAL_INPUT: &str = "local document input";
const STALE_URL: &str = "https://example.test/stale.html";

#[test]
fn workspace_restore_invalidates_unscoped_response_without_stealing_local_state()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let workspace_root = directory.path().to_path_buf();
    let local_path = workspace_root.join("local.html");
    let local_html = "<html><body>restored local document</body></html>";
    std::fs::write(&local_path, local_html)?;
    let mut app = make_app();
    configure_restore(&mut app, &workspace_root, &local_path);
    let sender = queue_request(&mut app, None, STALE_URL);
    let owner = app.state.url_tab.selection_owner.clone();
    app.state.url_tab.pending_url_requests[0].selection_owner = Some(owner);
    let ctx = egui::Context::default();
    restore_workspace(&mut app, workspace_root, &ctx);
    assert_restored_local(&app, &local_path, local_html);
    let status = Some(("local status".to_owned(), StatusType::Info));
    app.state.url_tab.input = LOCAL_INPUT.to_owned();
    app.state.layout.status_message = status.clone();
    sender.send(Ok(stale_html()))?;
    app.poll_url_source(&ctx);

    assert_local_state_after_stale(&app, &local_path, local_html, &status);
    Ok(())
}

fn restore_workspace(
    app: &mut crate::shell::KatanaApp,
    root: std::path::PathBuf,
    ctx: &egui::Context,
) {
    let workspace = katana_core::workspace::Workspace::new(&root, Vec::new());
    queued_explorer_result(app, ExplorerLoadType::Open, Ok(workspace), root);
    app.poll_explorer_load(ctx);
}

fn assert_local_state_after_stale(
    app: &crate::shell::KatanaApp,
    local_path: &Path,
    local_html: &str,
    status: &Option<(String, StatusType)>,
) {
    assert_eq!(app.state.active_path(), Some(local_path.to_path_buf()));
    assert_eq!(app.state.url_tab.input, LOCAL_INPUT);
    assert_eq!(app.state.layout.status_message.as_ref(), status.as_ref());
    assert_eq!(app.state.document.open_documents[0].buffer, local_html);
    assert_eq!(
        app.state
            .url_tab
            .tabs
            .iter()
            .find(|tab| tab.source.source_url == STALE_URL)
            .map(|tab| tab.source.raw_html.as_str()),
        Some("<html><body>stale HTTP response</body></html>")
    );
}

fn configure_restore(app: &mut crate::shell::KatanaApp, root: &Path, local: &Path) {
    let settings = &mut app.state.config.settings.settings_mut().workspace;
    settings.restore_session = true;
    settings.last_workspace = Some(root.display().to_string());
    settings.open_tabs = vec![local.display().to_string()];
    settings.active_tab_idx = Some(0);
    app.state.workspace.data = Some(katana_core::workspace::Workspace::new(root, Vec::new()));
}

fn assert_restored_local(app: &crate::shell::KatanaApp, path: &Path, body: &str) {
    assert_eq!(app.state.active_path(), Some(path.to_path_buf()));
    assert_eq!(app.state.document.open_documents.len(), 1);
    assert_eq!(app.state.document.open_documents[0].path, *path);
    assert_eq!(app.state.document.open_documents[0].buffer, body);
}
