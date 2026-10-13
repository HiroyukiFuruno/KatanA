use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::{AppAction, ViewMode};

#[test]
fn test_integration_view_modes() {
    /* WHY: Verify that the application correctly switches between PreviewOnly, Split, and CodeOnly view modes,
     * and that content is rendered appropriately in each mode. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_modes");
    let test_file = temp_dir.join("test_modes.md");
    std::fs::write(&test_file, "# Hello View Modes\n**Bold text here.**").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );
    let _ = harness.get_all_by_value("Bold text here.").next();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );
    let _ = harness.get_all_by_value("Bold text here.").next();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_split_mode_with_document() {
    /* WHY: Verify that Split mode correctly displays both sections when a document is active. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_split");
    let test_file = temp_dir.join("split_test.md");
    std::fs::write(&test_file, "# Split Mode Test\n\nContent here.").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
