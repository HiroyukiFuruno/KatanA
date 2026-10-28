use crate::app::{ActionOps, DocumentOps};
use crate::app_action::AppAction;
use crate::shell::KatanaApp;
use std::sync::Arc;

const OPEN_DOCUMENT_COUNT: usize = 3;

fn make_app() -> KatanaApp {
    let state = crate::app_state::AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    KatanaApp::new(state)
}

fn open_documents(
    app: &mut KatanaApp,
    directory: &std::path::Path,
) -> [std::path::PathBuf; OPEN_DOCUMENT_COUNT] {
    let paths = ["first.md", "middle.md", "last.md"].map(|name| {
        let path = directory.join(name);
        std::fs::write(&path, format!("# {name}\n")).expect("fixture must be written");
        path
    });
    for path in &paths {
        app.handle_select_document(path.clone(), true);
    }
    paths
}

#[test]
fn closing_tab_before_active_middle_keeps_the_same_document_active() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let directory = tempfile::tempdir().expect("tempdir must be created");
    let paths = open_documents(&mut app, directory.path());
    app.handle_select_document(paths[1].clone(), true);

    app.process_action(&ctx, AppAction::CloseDocument(0));

    assert_eq!(app.state.document.active_doc_idx, Some(0));
    assert_eq!(app.state.active_path(), Some(paths[1].clone()));
}

#[test]
fn closing_tab_before_active_last_keeps_the_same_document_active() {
    let mut app = make_app();
    let ctx = eframe::egui::Context::default();
    let directory = tempfile::tempdir().expect("tempdir must be created");
    let paths = open_documents(&mut app, directory.path());

    app.process_action(&ctx, AppAction::CloseDocument(0));

    assert_eq!(app.state.document.active_doc_idx, Some(1));
    assert_eq!(app.state.active_path(), Some(paths[2].clone()));
}
