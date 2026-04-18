use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use katana_ui::app_state::AppState;
use katana_ui::settings::tabs::icons::IconsTabOps;

#[test]
fn bug4_icons_advanced_settings_panel_must_show_table_content() {
    /* WHY: Regression test (Bug 4): Ensure that the Icon Advanced Settings bottom panel
     * correctly renders its internal table content when opened. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );

    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .build_ui(move |ui| {
            /* WHY: Inject the 'open' state into egui temp storage so the panel renders expanded. */
            ui.data_mut(|d| {
                d.insert_temp(egui::Id::new("icons_advanced_is_open"), true);
            });
            IconsTabOps::render_icons_tab_for_test(ui, &mut state);
        });
    for _ in 0..5 {
        harness.step();
    }

    /* WHY: Verify that the 'Advanced Settings' heading is present (panel is open). */
    let _heading = harness.get_by_label_contains("Advanced Settings");

    /* WHY: Verify that the 'Icon' column header is present, indicating the table is rendered. */
    let _icon_col_header = harness.get_by_label("Icon");
}
