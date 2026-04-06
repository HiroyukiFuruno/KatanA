use eframe::egui;
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use katana_ui::app_state::AppState;
use katana_ui::shell::KatanaApp;
use std::sync::Arc;

#[test]
fn foreground_blocker_prevents_background_clicks() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    // Simulate open palette
    state.command_palette.is_open = true;

    let app = KatanaApp::new(state);
    let ctx = egui::Context::default();

    // Check if background button consumes clicks
    let mut clicked = false;
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        // Evaluate blocker active logic and spawn shield
        if app.is_foreground_surface_active(ctx) {
            egui::Area::new("foreground_surface_blocker_1".into())
                .order(egui::Order::Middle)
                .fixed_pos(egui::pos2(0.0, 0.0))
                .interactable(true)
                .show(ctx, |ui| {
                    let rect = ctx.screen_rect();
                    ui.allocate_rect(rect, egui::Sense::click_and_drag());
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.button("Background Button");
            if response.clicked() {
                clicked = true;
            }
        });
    });

    // The first run doesn't dispatch a click.
    // Send a primary click to the center of the background.
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

    clicked = false;
    let _ = ctx.run(raw_input, |ctx| {
        if app.is_foreground_surface_active(ctx) {
            egui::Area::new("foreground_surface_blocker_2".into())
                .order(egui::Order::Middle)
                .fixed_pos(egui::pos2(0.0, 0.0))
                .interactable(true)
                .show(ctx, |ui| {
                    ui.allocate_rect(ctx.screen_rect(), egui::Sense::click_and_drag());
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // CentralPanel starts at roughly 0,0 but let's position a button explicitly at 50,50 to be sure
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
            let response = ui.put(rect, egui::Button::new("Background Button"));
            if response.clicked() {
                clicked = true;
            }
        });
    });

    // If blocker is active, the Background Button should not be clickable.
    assert!(
        !clicked,
        "Background button should not be clickable when modal is open"
    );
}
