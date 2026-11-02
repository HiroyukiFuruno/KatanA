use super::SplitMode;
use crate::app_state::AppState;
use crate::shell::KatanaApp;
use eframe::egui;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_platform::{PaneOrder, SplitDirection};

const VIEWPORT_WIDTH: f32 = 900.0;
const VIEWPORT_HEIGHT: f32 = 700.0;
const TOP_HEIGHT: f32 = 60.0;
const SIDEBAR_WIDTH: f32 = 180.0;
const STATUS_HEIGHT: f32 = 24.0;

fn app() -> KatanaApp {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(katana_core::document::Document::new_empty(
            std::path::PathBuf::from("split-regression.md"),
        ));
    state.document.active_doc_idx = Some(0);
    KatanaApp::new(state)
}

#[test]
fn split_background_stays_inside_remaining_panel_space() {
    for visuals in [egui::Visuals::dark(), egui::Visuals::light()] {
        for direction in [SplitDirection::Horizontal, SplitDirection::Vertical] {
            assert_split_background(&visuals, direction);
        }
    }
}

fn assert_split_background(visuals: &egui::Visuals, direction: SplitDirection) {
    let context = egui::Context::default();
    context.set_visuals(visuals.clone());
    let mut app = app();
    let mut remaining = egui::Rect::NOTHING;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        preceding_panels(ui);
        remaining = ui.available_rect_before_wrap();
        SplitMode::new(&context, &mut app, direction, PaneOrder::default()).show(ui);
    });
    output.textures_delta.clear();
    assert!(remaining.min.x > 0.0 && remaining.min.y > 0.0);
    assert!(remaining.max.y < VIEWPORT_HEIGHT);
    assert!(
        output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Rect(rect) if rect.fill == visuals.window_fill && rect.rect == remaining
        )),
        "split background must cover only remaining content: {remaining:?}"
    );
}

fn preceding_panels(ui: &mut egui::Ui) {
    egui::Panel::bottom("regression_status")
        .exact_size(STATUS_HEIGHT)
        .show_inside(ui, |_| {});
    egui::Panel::top("regression_top")
        .exact_size(TOP_HEIGHT)
        .show_inside(ui, |_| {});
    egui::Panel::left("regression_left")
        .exact_size(SIDEBAR_WIDTH)
        .show_inside(ui, |_| {});
}
