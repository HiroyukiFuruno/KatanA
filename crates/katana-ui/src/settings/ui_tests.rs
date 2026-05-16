use super::*;
use crate::app_state::SettingsTab;
use crate::preview_pane::PreviewPane;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;

#[test]
fn advanced_settings_table_is_rendered() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1024.0, 768.0))
        .build_ui(|ui| {
            let mut state = crate::app_state::AppState::new(
                AiProviderRegistry::new(),
                PluginRegistry::new(),
                katana_platform::SettingsService::default(),
                std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            );

            state.layout.show_settings = true;
            state.config.active_settings_tab = SettingsTab::Icons;

            ui.data_mut(|d| d.insert_temp(egui::Id::new("icons_advanced_is_open"), true));

            let mut preview_pane = PreviewPane::default();
            SettingsWindow::new(&mut state, &mut preview_pane).show(ui.ctx());
        });

    harness.run_steps(10);

    assert!(has_any_label(
        &harness,
        &[
            "Advanced Settings",
            "\u{9ad8}\u{5ea6}\u{306a}\u{8a2d}\u{5b9a}"
        ]
    ));
    assert!(has_any_label(
        &harness,
        &["Vendor", "\u{30d9}\u{30f3}\u{30c0}\u{30fc}"]
    ));
}

fn has_any_label(harness: &Harness<'_>, labels: &[&str]) -> bool {
    labels
        .iter()
        .any(|label| harness.query_by_label(label).is_some())
}
