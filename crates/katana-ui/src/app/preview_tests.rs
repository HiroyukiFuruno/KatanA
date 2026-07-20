use super::PreviewOps;
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
fn refresh_preview_routes_html_file_to_kdv_preview_surface() {
    let mut app = make_app();
    let source = r#"<html><body><h1>Title</h1><p>Body</p></body></html>"#;

    app.refresh_preview(std::path::Path::new("/tmp/index.html"), source);

    let pane = &app.tab_previews[0].pane;
    assert!(pane.has_html_browser());
    assert!(pane.sections.is_empty());
}

#[test]
fn full_refresh_preview_routes_htm_file_to_kdv_preview_surface() {
    let mut app = make_app();
    let source = r#"<details><summary>More</summary><p>Body</p></details>"#;

    app.full_refresh_preview(std::path::Path::new("/tmp/legacy.htm"), source, false, 4);

    let pane = &app.tab_previews[0].pane;
    assert!(pane.has_html_browser());
    assert!(pane.sections.is_empty());
    assert!(pane.render_rx.is_none());
    assert!(!pane.is_loading);
}
