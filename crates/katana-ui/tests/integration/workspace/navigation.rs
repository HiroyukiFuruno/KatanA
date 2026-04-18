use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

#[test]
fn test_integration_workspace_and_tabs() {
    /* WHY: Verify that opening a workspace populated with files correctly items in the file tree
     * and allows opening/closing tabs. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws");
    let test_file = temp_dir.join("test1.md");
    std::fs::write(&test_file, "# Hello Katana").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    let file_node = harness.get_all_by_value("file test1.md").next().unwrap();

    file_node.click();
    harness.step();
    harness.step();

    assert!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .active_doc_idx
            .is_some()
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    assert!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .active_doc_idx
            .is_none()
    );
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_workspace_with_subdirectory() {
    /* WHY: Verify that the workspace correctly scans and displays subdirectories and their contents. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_subdir");
    std::fs::create_dir_all(temp_dir.join("docs")).unwrap();
    std::fs::write(temp_dir.join("root.md"), "# Root").unwrap();
    std::fs::write(temp_dir.join("docs").join("inner.md"), "# Inner").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    let _ = harness.get_by_label("dir docs");
    let _ = harness.get_by_label("file root.md");

    let _ = std::fs::remove_dir_all(&temp_dir);
}
