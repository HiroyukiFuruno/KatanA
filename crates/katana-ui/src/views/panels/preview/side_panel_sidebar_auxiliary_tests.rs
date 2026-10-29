use super::*;
use crate::app_state::AppState;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};

const TEST_VIEWPORT_SIZE: [f32; 2] = [320.0, 240.0];

#[path = "side_panel_sidebar_auxiliary/tests/reload.rs"]
mod reload_tests;

fn test_app() -> crate::shell::KatanaApp {
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    crate::shell::KatanaApp::new(state)
}

fn pointer_input(pos: egui::Pos2, pressed: bool) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(TEST_VIEWPORT_SIZE[0], TEST_VIEWPORT_SIZE[1]),
        )),
        events: vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    }
}

fn render_action_button_for_test(
    ctx: &egui::Context,
    app: &mut crate::shell::KatanaApp,
    input: egui::RawInput,
    action: AppAction,
) -> egui::Rect {
    let mut button_rect = None;
    let mut output = ctx.run_ui(input, |ui| {
        let mut panels = PreviewSidePanels::new(app);
        let response = panels.render_action_button(
            ui,
            SidebarButton {
                available: true,
                icon: crate::Icon::Refresh,
                active: false,
                label: "Refresh",
                shortcut: None,
                action: action.clone(),
            },
        );
        button_rect = Some(response.rect);
    });
    output.textures_delta.clear();
    button_rect.expect("refresh button geometry")
}

#[test]
fn render_action_button_pointer_click_queues_manual_refresh() {
    let mut app = test_app();
    let ctx = egui::Context::default();
    let action = AppAction::RefreshDocument { is_manual: true };
    let pointer =
        render_action_button_for_test(&ctx, &mut app, egui::RawInput::default(), action.clone())
            .center();
    render_action_button_for_test(&ctx, &mut app, pointer_input(pointer, true), action.clone());
    render_action_button_for_test(&ctx, &mut app, pointer_input(pointer, false), action);
    assert!(matches!(
        app.pending_action,
        AppAction::RefreshDocument { is_manual: true }
    ));
}

fn render_primary_controls_for_test(
    ctx: &egui::Context,
    app: &mut crate::shell::KatanaApp,
    input: egui::RawInput,
) {
    let mut output = ctx.run_ui(input, |ui| {
        PreviewSidePanels::new(app).render_primary_controls(ui, true, true);
    });
    output.textures_delta.clear();
}

#[test]
fn primary_sidebar_pointer_click_queues_manual_refresh() {
    let mut app = test_app();
    let ctx = egui::Context::default();
    app.pending_action = AppAction::None;
    assert!(matches!(app.pending_action, AppAction::None));
    let pointer = render_action_button_for_test(
        &ctx,
        &mut app,
        egui::RawInput::default(),
        AppAction::RefreshDocument { is_manual: true },
    )
    .center();
    render_primary_controls_for_test(&ctx, &mut app, pointer_input(pointer, true));
    render_primary_controls_for_test(&ctx, &mut app, pointer_input(pointer, false));
    assert!(matches!(
        app.pending_action,
        AppAction::RefreshDocument { is_manual: true }
    ));
}
