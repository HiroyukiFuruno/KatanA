use super::ImageDocumentOps;
use crate::app::DocumentOps;

struct CanvasAppTest;

impl CanvasAppTest {
    fn app(path: &std::path::Path) -> crate::shell::KatanaApp {
        let state = crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = crate::shell::KatanaApp::new(state);
        app.state
            .document
            .open_documents
            .push(ImageDocumentOps::load_or_create(&app.fs, path).expect("Canvas reference"));
        app.state.document.active_doc_idx = Some(0);
        app
    }
}

#[test]
fn opening_canvas_creates_a_reference_without_synchronous_source_read() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("graph.canvas");
    std::fs::write(&path, [0xff]).expect("invalid UTF8 fixture");
    let document =
        ImageDocumentOps::load_or_create(&katana_platform::FilesystemService::new(), &path)
            .expect("source validation belongs to asynchronous Canvas intake");
    assert!(document.is_reference);
    assert!(document.buffer.is_empty());
    assert_eq!(document.path, path);
}

#[test]
fn read_only_canvas_rejects_buffer_edit_actions() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("graph.canvas");
    std::fs::write(&path, "{\"nodes\":[]}").expect("Canvas fixture");
    let mut app = CanvasAppTest::app(&path);
    app.handle_update_buffer("replacement".to_owned());
    assert!(app.state.document.open_documents[0].buffer.is_empty());
    assert!(!app.state.document.open_documents[0].is_dirty);
}

#[test]
fn read_only_canvas_save_never_overwrites_the_source_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("graph.canvas");
    let original = "{\"nodes\":[]}";
    std::fs::write(&path, original).expect("Canvas fixture");
    let mut app = CanvasAppTest::app(&path);
    app.state.document.open_documents[0].buffer = "replacement".to_owned();
    app.state.document.open_documents[0].is_dirty = true;
    app.handle_save_document();
    assert_eq!(std::fs::read_to_string(&path).expect("source"), original);
}

#[test]
fn opening_a_background_canvas_does_not_change_the_active_markdown_view_mode() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let active_path = directory.path().join("main.md");
    let canvas_path = directory.path().join("graph.canvas");
    std::fs::write(&active_path, "# Active Markdown").expect("Markdown fixture");
    std::fs::write(&canvas_path, "{}").expect("Canvas fixture");
    let mut app = CanvasAppTest::app(&active_path);
    let mode = crate::state::document::ViewMode::Split;
    app.state.set_active_view_mode(mode);
    app.handle_select_document(canvas_path, false);
    assert_eq!(
        app.state.active_path().as_deref(),
        Some(active_path.as_path())
    );
    assert_eq!(app.state.active_view_mode(), mode);
}
