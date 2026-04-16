use egui_kittest::Harness;
use egui::vec2;
use egui_kittest::kittest::Queryable;

#[test]
fn test_regression_advanced_settings_content_visibility_red() {
    let mut harness = Harness::builder()
        .with_size(vec2(800.0, 600.0))
        .build_ui(|ui| {
            // Mock state
            let mut state = crate::app_state::AppState::default();
            let i18n = crate::i18n::I18nMessages::default();
            let mut icon_settings = katana_platform::settings::types::icon::IconSettings::default();
            let mut settings_changed = false;
            
            // Force "Open" state for the panel
            let mut is_open = true;

            katana_ui::settings::tabs::icons::panels::IconsPanelsOps::render_panels(
                ui,
                &mut state,
                &i18n,
                &mut is_open,
                &mut icon_settings,
                &mut settings_changed,
            );
        });

    harness.run();

    // EXPECTATION: "高度な設定" heading (from panels.rs) should be visible.
    // If THIS fails, the panel itself isn't rendering.
    assert!(harness.get_by_label("高度な設定").is_some(), "Advanced settings heading should be visible");

    // RED CASE: "アイコン" label (from table.rs header) should be visible,
    // but we expect it to be MISSING or HEIGHT 0 due to the bug.
    // Using try_get_by_label or checking size.
    let icon_label = harness.get_by_label("アイコン");
    
    assert!(
        icon_label.is_none(), 
        "RED REPRODUCTION: The internal table content ('アイコン') should NOT be visible due to the layout bug. If it IS found, the RED reproduction failed."
    );
}
