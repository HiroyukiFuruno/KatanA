use super::*;
use crate::app_state::AppState;
use crate::shell::KatanaApp;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use std::sync::Arc;

fn app() -> KatanaApp {
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        Default::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    KatanaApp::new(state)
}

fn assert_page_focus_request(ctx: &egui::Context, path: &std::path::Path) {
    assert_eq!(focus_page(ctx, path), Some(true));
}

fn focus_page(ctx: &egui::Context, owner_path: &std::path::Path) -> Option<bool> {
    let mut result = None;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(100.0, 100.0),
        )),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| {
        let response = ui.interact(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)),
            egui::Id::new(("document-page-navigation", 0_u64)),
            egui::Sense::focusable_noninteractive(),
        );
        result = crate::app::preview::keyboard_focus::PageNavigationFocusOps::focus_if_activated(
            ui.ctx(),
            &response,
            owner_path,
        );
    });
    output.textures_delta.clear();
    drop(output);
    result
}

#[test]
fn file_dialog_open_selection_arms_page_navigation_focus() {
    for use_current_workspace in [false, true] {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("deck.pdf");
        std::fs::write(&path, b"%PDF-1.7\n").expect("write fixture");
        let ctx = egui::Context::default();
        let mut app = app();
        if use_current_workspace {
            app.state.workspace.data = Some(katana_core::workspace::Workspace::new(
                directory.path(),
                Vec::new(),
            ));
        }
        let action = if use_current_workspace {
            AppAction::PickOpenFileInCurrentWorkspace
        } else {
            AppAction::PickOpenFileInNewWorkspace
        };
        app.handle_file_dialog_action(&ctx, path.clone(), action);
        assert_page_focus_request(&ctx, &path);
    }
}

#[test]
fn invalid_file_dialog_selection_clears_a_pending_page_focus_request() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let valid_path = directory.path().join("deck.pdf");
    let invalid_path = directory.path().join("missing.pdf");
    std::fs::write(&valid_path, b"%PDF-1.7\n").expect("write fixture");
    let ctx = egui::Context::default();
    crate::app::preview::keyboard_focus::PageNavigationFocusOps::arm_for_selection(
        &ctx,
        &valid_path,
    );
    let mut app = app();
    app.handle_file_dialog_action(&ctx, invalid_path, AppAction::PickOpenFileInNewWorkspace);

    assert_eq!(focus_page(&ctx, &valid_path), None);
}
