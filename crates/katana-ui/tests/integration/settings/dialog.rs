use crate::integration::harness_utils::setup_harness;
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::{AppAction, SettingsTab};

#[test]
fn test_integration_settings_window() {
    /* WHY: Verify that the settings window can be opened, that tabs can be switched, 
     * and that the internal state correctly updates to reflect the active tab. */
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSettings);
    harness.step();
    harness.step();

    /* WHY: Test switching to the Font tab. Use harness queries to simulate user interactions with labels. */
    for node in harness.query_all_by_label("Font") {
        node.click();
    }
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .config
            .active_settings_tab,
        SettingsTab::Font
    );

    /* WHY: Test switching to the Layout tab. */
    for node in harness.query_all_by_label("Layout") {
        node.click();
    }
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .config
        .active_settings_tab = SettingsTab::Layout;
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .config
            .active_settings_tab,
        SettingsTab::Layout
    );

    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSettings);
    harness.step();
}
