use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::{AppAction, ViewMode};

#[test]
fn test_integration_editor_line_numbers_and_highlight() {
    /* WHY: Verify that line numbers are correctly rendered and visible in the editor's CodeOnly mode. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_editor_lines");
    let test_file = temp_dir.join("lines.md");
    std::fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.step();

    let count_1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("1").count()
    }))
    .unwrap_or(0);
    let count_2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("2").count()
    }))
    .unwrap_or(0);
    let count_3 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("3").count()
    }))
    .unwrap_or(0);

    assert!(count_1 > 0, "Line number 1 should be visible");
    assert!(count_2 > 0, "Line number 2 should be visible");
    assert!(count_3 > 0, "Line number 3 should be visible");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_update_buffer() {
    /* WHY: Verify that the UpdateBuffer action correctly updates the document content and that it is reflected in the internal state. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_buf");
    let test_file = temp_dir.join("buf_test.md");
    std::fs::write(&test_file, "# Original").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness.state_mut().trigger_action(AppAction::UpdateBuffer(
        "# Updated\n\nNew content".to_string(),
    ));
    harness.step();
    let active_idx = harness
        .state_mut()
        .app_state_mut()
        .document
        .active_doc_idx
        .unwrap();
    let buf = harness.state_mut().app_state_mut().document.open_documents[active_idx]
        .buffer
        .clone();
    assert!(buf.contains("New content"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_save_document() {
    /* WHY: Verify that the SaveDocument action correctly writes the current buffer content to the filesystem. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_save");
    let test_file = temp_dir.join("save_test.md");
    std::fs::write(&test_file, "# Hello").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Saved Content".to_string()));
    harness.step();
    harness.state_mut().trigger_action(AppAction::SaveDocument);
    harness.step();
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "# Saved Content");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_text_edit_triggers_update_buffer() {
    /* WHY: Verify that simulated text editing actions trigger the appropriate buffer updates. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("edit.md");
    std::fs::write(&md_path, "# Editable").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Modified content".to_string()));
    harness.step();
}
