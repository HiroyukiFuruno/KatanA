use crate::integration::harness_utils::{setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

#[test]
fn test_file_entry_label_is_left_aligned() {
    /* WHY: Verify that file entry labels in the explorer are left-aligned
     * (their width should be text-width, not full-row-width which would push icons to the right). */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("alignment.md"), "# Alignment").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    let nodes: Vec<_> = harness.query_all_by_label("file alignment.md").collect();
    assert!(!nodes.is_empty());

    let node = &nodes[0];
    let label_rect = node.rect();
    let label_width = label_rect.width();
    assert!(
        label_width < 176.0,
        "File entry label width must be short (indicates bug in row width allocation)"
    );
}

#[test]
fn test_file_entry_click_opens_document() {
    /* WHY: Verify that clicking the text area of a file entry correctly
     * triggers opening that document. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("clickable.md"), "# Clickable").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    let nodes: Vec<_> = harness.query_all_by_label("file clickable.md").collect();
    nodes[0].click();
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
}
