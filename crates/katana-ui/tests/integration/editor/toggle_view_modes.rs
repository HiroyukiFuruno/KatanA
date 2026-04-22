use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use katana_ui::app_state::{AppAction, ViewMode};

#[test]
fn test_integration_toggle_view_modes() {
    /* WHY: Verify that ToggleSplitMode and ToggleCodePreview actions behave as specified.
     * ToggleSplitMode: Always sets mode to Split.
     * ToggleCodePreview cycle: Split -> Preview -> Code -> Preview -> Code ... */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toggle_modes");
    let test_file = temp_dir.join("test_toggle.md");
    std::fs::write(&test_file, "# Toggle Test").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    // 1. Initial mode should be PreviewOnly (default)
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );

    // 2. ToggleSplitMode -> Always Split
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSplitMode);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );

    // 3. ToggleCodePreview when in Split -> PreviewOnly
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCodePreview);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );

    // 4. ToggleCodePreview when in PreviewOnly -> CodeOnly
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCodePreview);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );

    // 5. ToggleCodePreview when in CodeOnly -> PreviewOnly
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCodePreview);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );

    // 6. Test ToggleSplitMode again from CodeOnly
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCodePreview); // to CodeOnly
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSplitMode);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
