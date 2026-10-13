use super::*;
use crate::app_state::AppState;
use crate::settings::tabs::preset_widget::PresetWidgetResponse;
use egui_kittest::Harness;
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use std::cell::Cell;

#[test]
fn advanced_button_opens_separate_advanced_view() {
    let recorded = Cell::new(false);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(320.0, 120.0))
        .build_ui(|ui| {
            let mut state = AppState::new(
                AiProviderRegistry::new(),
                PluginRegistry::new(),
                katana_platform::SettingsService::default(),
                std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            );
            let response = PresetWidgetResponse {
                selected: None,
                save_clicked: false,
                revert_clicked: false,
                advanced_clicked: true,
            };
            let mut is_advanced_open = false;
            ThemePresetControlsOps::handle_response(
                ui,
                &mut state,
                response,
                &mut is_advanced_open,
            );

            recorded.set(is_advanced_open);
        });

    harness.run_steps(1);
    assert!(recorded.get());
}
