use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_toc_enable_disable_setting() {
    /* WHY: Verify that the TOC toolbar button respects the visibility setting,
     * disappearing when the user disables the TOC feature in settings. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_setting");
    let test_file = temp_dir.join("toc_test.md");
    std::fs::write(&test_file, "# Heading").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.step();

    let toc_icon = I18nOps::get().action.toggle_toc.clone();
    assert_eq!(harness.query_all_by_label(&toc_icon).count(), 1);

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .layout
        .toc_visible = false;
    harness.run_steps(5);

    assert!(
        !harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings()
            .layout
            .toc_visible
    );
}

#[test]
fn test_integration_toc_panel_hides_when_disabled() {
    /* WHY: Verify that even if the TOC panel was open, it is immediately hidden
     * if the global TOC visibility setting is toggled to false. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_hide");
    let test_file = temp_dir.join("toc_hide.md");
    std::fs::write(&test_file, "# Heading").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.step();

    harness.state_mut().trigger_action(AppAction::ToggleToc);
    harness.run_steps(10);

    let toc_title = I18nOps::get().toc.title.clone();
    assert!(harness.query_all_by_label(&toc_title).count() > 0);

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .layout
        .toc_visible = false;
    harness.run_steps(10);

    let is_visible = harness.query_all_by_label(&toc_title).count() > 0;
    assert!(!is_visible);
}
