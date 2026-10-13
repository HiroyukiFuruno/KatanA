use super::{DocumentOps, ExportOps};
use crate::shell::KatanaApp;
use std::sync::Arc;

fn make_app() -> KatanaApp {
    let state = crate::app_state::AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    KatanaApp::new(state)
}

#[test]
fn export_document_rejects_html_active_document() {
    let mut app = make_app();
    let dir = tempfile::tempdir().expect("create temp directory");
    let path = dir.path().join("index.html");
    std::fs::write(&path, "<h1>Title</h1>").expect("write html fixture");

    app.handle_select_document(path.clone(), true);
    app.handle_export_document(
        &egui::Context::default(),
        crate::app_state::ExportFormat::Html,
    );

    assert!(app.export_tasks.is_empty());
    let Some((message, status_type)) = app.state.layout.status_message else {
        panic!("status message should be set");
    };
    assert_eq!(status_type, crate::app_state::StatusType::Warning);
    assert!(message.contains("index.html"));
}
