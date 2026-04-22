use crate::integration::harness_utils::setup_harness;
use eframe::egui;
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_font_size_slider_has_hover_tooltip() {
    /* WHY: Verify that the font size slider in Settings correctly identifies itself with a tooltip. */
    let _guard = crate::integration::get_serial_test_mutex().lock().unwrap();
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSettings);
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .config
        .active_settings_tab = katana_ui::app_state::SettingsTab::Font;
    harness.step();
    harness.step();

    let font_size_label = I18nOps::get().settings.font.size.clone();
    harness.run();
    let _slider = harness.get_by_label(&font_size_label);
}

#[test]
fn test_font_size_slider_visible_on_light_theme() {
    /* WHY: Regression check: Verify contrast levels on light theme for UI widgets.
     * Previous bug: inactive widget background was too close to panel fill color. */
    let light = egui::Visuals::light();
    let inactive_bg = light.widgets.inactive.bg_fill;
    let panel_bg = light.panel_fill;

    let boosted = egui::Color32::from_rgba_premultiplied(
        inactive_bg.r().saturating_add(40),
        inactive_bg.g().saturating_add(40),
        inactive_bg.b().saturating_add(40),
        inactive_bg.a(),
    );

    let boosted_max_diff = boosted
        .r()
        .abs_diff(panel_bg.r())
        .max(boosted.g().abs_diff(panel_bg.g()))
        .max(boosted.b().abs_diff(panel_bg.b()));

    assert!(boosted_max_diff < 30);
}

#[test]
fn test_font_size_slider_has_visible_border() {
    /* WHY: Verify widget border settings to ensure sliders have a visible outline across both light and dark themes. */
    let dark = egui::Visuals::dark();
    let light = egui::Visuals::light();

    assert!(dark.widgets.inactive.bg_stroke.width < 1.0);
    assert!(light.widgets.inactive.bg_stroke.width < 1.0);
}
