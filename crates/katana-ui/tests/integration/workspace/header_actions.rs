use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

#[test]
fn test_explorer_header_creation_buttons_open_root_modals() {
    /* WHY: Header creation buttons must reuse the same create modal state as context-menu creation,
     * with the workspace root as the parent directory. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_header_create");
    std::fs::write(temp_dir.join("anchor.md"), "# Anchor").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness.step();
    harness.step();

    harness.get_by_label("file+").click();
    harness.step();
    harness.step();
    let modal = harness
        .state_mut()
        .app_state_mut()
        .layout
        .create_fs_node_modal
        .clone()
        .expect("File creation modal should open");
    assert_eq!(modal.0, temp_dir);
    assert!(!modal.3, "File creation button must open file mode");

    harness
        .state_mut()
        .app_state_mut()
        .layout
        .create_fs_node_modal = None;
    harness.get_by_label("folder+").click();
    harness.step();
    harness.step();
    let modal = harness
        .state_mut()
        .app_state_mut()
        .layout
        .create_fs_node_modal
        .clone()
        .expect("Folder creation modal should open");
    assert_eq!(modal.0, temp_dir);
    assert!(modal.3, "Folder creation button must open directory mode");

    let _ = std::fs::remove_dir_all(&temp_dir);
}
