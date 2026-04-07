use eframe::egui;
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use katana_ui::app_state::AppState;
use katana_ui::shell::KatanaApp;
use std::sync::Arc;

fn make_blocked_state() -> AppState {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.command_palette.is_open = true;
    state
}

/// Verifies that `ui.set_enabled(false)` suppresses click on background widgets.
#[test]
fn disabled_ui_prevents_background_clicks() {
    let state = make_blocked_state();
    let app = KatanaApp::new(state);
    let ctx = egui::Context::default();

    let is_blocked = app.is_foreground_surface_active(&ctx);

    // Frame 1: seed the layout
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!is_blocked);
            let _ = ui.button("Background Button");
        });
    });

    // Frame 2: send a click
    let mut raw_input = egui::RawInput::default();
    raw_input
        .events
        .push(egui::Event::PointerMoved(egui::pos2(50.0, 50.0)));
    raw_input.events.push(egui::Event::PointerButton {
        pos: egui::pos2(50.0, 50.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    raw_input.events.push(egui::Event::PointerButton {
        pos: egui::pos2(50.0, 50.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });

    let mut clicked = false;
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!is_blocked);
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
            let response = ui.put(rect, egui::Button::new("Background Button"));
            if response.clicked() {
                clicked = true;
            }
        });
    });

    assert!(
        !clicked,
        "Background button should not be clickable when UI is disabled"
    );
}

/// Verifies that `ui.set_enabled(false)` suppresses hover on background widgets.
#[test]
fn disabled_ui_prevents_background_hover() {
    let state = make_blocked_state();
    let app = KatanaApp::new(state);
    let ctx = egui::Context::default();

    let is_blocked = app.is_foreground_surface_active(&ctx);

    // Frame 1: seed the layout
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!is_blocked);
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
            ui.put(rect, egui::Button::new("Hoverable Button"));
        });
    });

    // Frame 2: move pointer over the background button
    let mut raw_input = egui::RawInput::default();
    raw_input
        .events
        .push(egui::Event::PointerMoved(egui::pos2(50.0, 50.0)));

    let mut hovered = false;
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!is_blocked);
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
            let response = ui.put(rect, egui::Button::new("Hoverable Button"));
            if response.hovered() {
                hovered = true;
            }
        });
    });

    assert!(
        !hovered,
        "Background button should not receive hover when UI is disabled"
    );
}
