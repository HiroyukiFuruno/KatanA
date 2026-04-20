use katana_ui::app_state::AppAction;
use egui_kittest::kittest::Queryable;
use crate::integration::harness_utils::{setup_harness, fresh_temp_dir, wait_for_workspace_load};

#[test]
fn test_integration_workspace_and_tabs_navigation() {
    /* WHY: Verify that opening a file from the explorer correctly creates a tab, 
     * activates it, and that closing the tab correctly updates the active document state. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_nav");
    let test_file = temp_dir.join("test1.md");
    std::fs::write(&test_file, "# Hello Katana").unwrap();

    harness.state_mut().trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);

    let file_node = harness.get_all_by_value("file test1.md").next().unwrap();
    file_node.click();
    harness.run_steps(10);

    assert!(harness.state_mut().app_state_mut().document.active_doc_idx.is_some());

    harness.state_mut().trigger_action(AppAction::CloseDocument(0));
    harness.run_steps(5);

    assert!(harness.state_mut().app_state_mut().document.active_doc_idx.is_none());
}

#[test]
fn test_integration_open_multiple_documents_and_switch() {
    /* WHY: Verify that multiple documents can be opened and switched between, 
     * and that the UI correctly updates the displayed content. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_multi");
    let f1 = temp_dir.join("f1.md");
    let f2 = temp_dir.join("f2.md");
    std::fs::write(&f1, "# Doc 1").unwrap();
    std::fs::write(&f2, "# Doc 2").unwrap();

    harness.state_mut().trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);

    harness.state_mut().trigger_action(AppAction::SelectDocument(f1.canonicalize().unwrap()));
    harness.run_steps(10);
    harness.state_mut().trigger_action(AppAction::SelectDocument(f2.canonicalize().unwrap()));
    harness.run_steps(10);

    assert_eq!(harness.state_mut().app_state_mut().document.documents.len(), 2);
    assert_eq!(harness.state_mut().app_state_mut().document.active_doc_idx, Some(1));
}
