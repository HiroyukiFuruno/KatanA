use super::*;
use crate::state::ViewMode;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use std::sync::Arc;

fn app_with_active_path(path: &str) -> KatanaApp {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(path));
    state.document.active_doc_idx = Some(0);
    KatanaApp::new(state)
}

#[test]
fn direct_view_tools_dispatch_is_rejected_for_office_and_html_documents() {
    for path in [
        "report.html",
        "REPORT.HTM",
        "report.docx",
        "book.xlsx",
        "deck.pptx",
    ] {
        for action in [AppAction::ToggleSplitMode, AppAction::ToggleCodePreview] {
            let mut app = app_with_active_path(path);
            app.state.set_active_view_mode(ViewMode::CodeOnly);
            app.dispatch_secondary(&egui::Context::default(), action.clone());
            assert_eq!(
                app.state.active_view_mode(),
                ViewMode::CodeOnly,
                "{action:?} must be rejected for {path}"
            );
        }
    }
}

#[test]
fn direct_view_tools_dispatch_remains_available_for_pdf_and_markdown_documents() {
    for path in ["report.pdf", "readme.md"] {
        let mut split_app = app_with_active_path(path);
        split_app.state.set_active_view_mode(ViewMode::CodeOnly);
        split_app.dispatch_secondary(&egui::Context::default(), AppAction::ToggleSplitMode);
        assert_eq!(split_app.state.active_view_mode(), ViewMode::Split);

        let mut preview_app = app_with_active_path(path);
        preview_app.state.set_active_view_mode(ViewMode::CodeOnly);
        preview_app.dispatch_secondary(&egui::Context::default(), AppAction::ToggleCodePreview);
        assert_eq!(preview_app.state.active_view_mode(), ViewMode::PreviewOnly);
    }
}

#[test]
fn direct_set_view_mode_rejects_split_and_code_only_for_html_and_office_documents() {
    for path in [
        "report.html",
        "REPORT.HTM",
        "report.docx",
        "book.xlsx",
        "deck.pptx",
    ] {
        for mode in [ViewMode::Split, ViewMode::CodeOnly] {
            let mut app = app_with_active_path(path);
            app.dispatch_secondary(&egui::Context::default(), AppAction::SetViewMode(mode));
            assert_eq!(
                app.state.active_view_mode(),
                ViewMode::PreviewOnly,
                "direct SetViewMode({mode:?}) must be rejected for {path}"
            );
        }

        let mut app = app_with_active_path(path);
        app.state.set_active_view_mode(ViewMode::CodeOnly);
        app.dispatch_secondary(
            &egui::Context::default(),
            AppAction::SetViewMode(ViewMode::PreviewOnly),
        );
        assert_eq!(app.state.active_view_mode(), ViewMode::PreviewOnly);
    }
}

#[test]
fn direct_set_view_mode_preserves_all_modes_for_markdown_documents() {
    for mode in [ViewMode::PreviewOnly, ViewMode::CodeOnly, ViewMode::Split] {
        let mut app = app_with_active_path("readme.md");
        app.dispatch_secondary(&egui::Context::default(), AppAction::SetViewMode(mode));
        assert_eq!(
            app.state.active_view_mode(),
            mode,
            "direct SetViewMode({mode:?}) must remain available for Markdown"
        );
    }
}
