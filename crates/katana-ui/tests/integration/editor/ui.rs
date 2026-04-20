use katana_ui::app_state::{AppAction, ViewMode};
use egui_kittest::kittest::Queryable;
use crate::integration::harness_utils::{setup_harness, fresh_temp_dir, wait_for_workspace_load};

#[test]
fn test_integration_view_modes() {
    /* WHY: Verify that the application correctly switches between Split, 
     * PreviewOnly, and CodeOnly modes, and that the UI state reflects these changes. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_modes");
    let test_file = temp_dir.join("test_modes.md");
    std::fs::write(&test_file, "# Hello View Modes\n**Bold text here.**").unwrap();

    harness.state_mut().trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness.state_mut().trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.step();

    harness.state_mut().app_state_mut().set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().active_view_mode(), ViewMode::PreviewOnly);

    harness.state_mut().app_state_mut().set_active_view_mode(ViewMode::Split);
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().active_view_mode(), ViewMode::Split);

    harness.state_mut().app_state_mut().set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().active_view_mode(), ViewMode::CodeOnly);
}

#[test]
fn test_integration_editor_line_numbers_visibility() {
    /* WHY: Verify that line numbers are rendered in the editor when a document is open. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_editor_lines");
    let test_file = temp_dir.join("lines.md");
    std::fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();

    harness.state_mut().trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);
    harness.state_mut().trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.run_steps(10);

    harness.state_mut().app_state_mut().set_active_view_mode(ViewMode::CodeOnly);
    harness.run_steps(5);

    let count_1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| harness.query_all_by_label("1").count())).unwrap_or(0);
    assert!(count_1 > 0, "Line number 1 should be visible");
}

#[test]
fn test_integration_update_buffer() {
    /* WHY: Verify that modifying the editor buffer correctly updates the internal state 
     * and that these changes are reflected in the preview pane (if visible). */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_buf");
    let test_file = temp_dir.join("buf_test.md");
    std::fs::write(&test_file, "# Original").unwrap();

    harness.state_mut().trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);
    harness.state_mut().trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.run_steps(10);

    // Simulate typing " Updated"
    harness.state_mut().app_state_mut().trigger_action(AppAction::UpdateBuffer("# Original Updated".to_string()));
    harness.run_steps(10);

    assert!(harness.query_all_by_label("Original Updated").count() > 0);
}
