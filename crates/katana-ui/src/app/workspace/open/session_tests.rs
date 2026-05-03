#![allow(clippy::unwrap_used)]

use super::session::WorkspaceOpenSessionOps;
use crate::app_state::AppState;
use crate::shell::KatanaApp;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};

fn make_app() -> KatanaApp {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
        katana_platform::workspace::InMemoryWorkspaceRepository::default(),
    ));
    KatanaApp::new(state)
}

fn make_workspace(root: &std::path::Path) -> katana_core::workspace::Workspace {
    katana_core::workspace::Workspace::new(root, Vec::new())
}

#[test]
fn apply_session_tabs_restores_active_image_as_reference_document() {
    let mut app = make_app();
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("diagram.png");
    std::fs::write(&image_path, b"\x89PNG\r\n\x1a\n").unwrap();
    app.state.workspace.data = Some(make_workspace(dir.path()));

    WorkspaceOpenSessionOps::apply_session_tabs(
        &mut app,
        vec![(image_path.display().to_string(), false)],
        Some(0),
        dir.path().display().to_string(),
    );

    let doc = app.state.document.open_documents.first().unwrap();
    assert_eq!(doc.path, image_path);
    assert!(doc.is_loaded);
    assert!(doc.is_reference);
    assert_eq!(doc.buffer, format!("![](file://{})", image_path.display()));
    assert_eq!(app.state.document.active_doc_idx, Some(0));
}

#[test]
fn load_active_session_document_keeps_drawio_as_source_document() {
    let app = make_app();
    let dir = tempfile::tempdir().unwrap();
    let drawio_path = dir.path().join("diagram.drawio");
    let source = "<mxfile><diagram><mxGraphModel /></diagram></mxfile>";
    std::fs::write(&drawio_path, source).unwrap();

    let doc =
        WorkspaceOpenSessionOps::load_active_session_document(&app, &drawio_path, true).unwrap();

    assert_eq!(doc.path, drawio_path);
    assert!(doc.is_loaded);
    assert!(!doc.is_reference);
    assert_eq!(doc.buffer, source);
    assert!(doc.is_pinned);
}
