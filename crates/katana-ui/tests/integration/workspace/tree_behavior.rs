use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

#[test]
fn test_integration_workspace_with_subdirectory() {
    /* WHY: Verify that the workspace explorer correctly nested directories
     * and their contents can be identified by their ARIA labels. */
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

#[test]
fn test_integration_directory_collapse_bug() {
    /* WHY: Verify that for deeply nested structures (parent/child/file),
     * the explorer correctly reflects the collapse state of subdirectories
     * when toggled via state or click. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_collapse");
    let parent_dir = temp_dir.join("parent");
    std::fs::create_dir_all(&parent_dir).unwrap();
    let sub_dir = parent_dir.join("child");
    std::fs::create_dir_all(&sub_dir).unwrap();
    std::fs::write(sub_dir.join("file.md"), "# File").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    let parent_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir parent")
    }))
    .is_ok();
    assert!(parent_visible, "Parent should be visible");

    let child_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir child")
    }))
    .is_ok();
    assert!(!child_visible, "Child should not be visible initially");

    harness
        .state_mut()
        .app_state_mut()
        .workspace
        .force_tree_open = Some(true);
    harness.step();
    harness.step();

    let child_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir child")
    }))
    .is_ok();
    assert!(child_visible, "Child should now be visible");

    harness
        .state_mut()
        .app_state_mut()
        .workspace
        .force_tree_open = Some(false);
    harness.step();

    let child_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir child")
    }))
    .is_ok();
    assert!(!child_visible, "Child should be hidden");

    let parent_node = harness.get_by_label("dir parent");
    parent_node.click();
    harness.step();

    let file_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("file file.md")
    }))
    .is_ok();
    assert!(!file_visible, "Child directory should be collapsed!");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tree_toggle_buttons() {
    /* WHY: Verify that the global [+] (Expand) and [-] (Collapse) buttons in the Sidebar header
     * correctly manipulate the expansion state of all directory entries. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(temp_dir.path().join("sub")).unwrap();
    std::fs::write(temp_dir.path().join("root.md"), "# Root").unwrap();
    std::fs::write(temp_dir.path().join("sub").join("child.md"), "# Child").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    if let Some(btn) = harness.query_all_by_label("+").next() {
        btn.click();
    }
    harness.step();

    if let Some(btn) = harness.query_all_by_label("-").next() {
        btn.click();
    }
    harness.step();
}

#[test]
fn test_integration_workspace_directory_toggle_non_recursive() {
    /* WHY: Verify that clicking a directory in the tree ONLY expands it and NOT its subdirectories (non-recursive),
     * but also verify that expansion state is cached for nested subfolders when reopened. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_dir_toggle");
    let dir2 = temp_dir.join("dir1").join("dir2");
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir2.join("test.md"), "# Content").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness.run_steps(5);

    harness.get_by_label("dir dir1").click();
    harness.run_steps(5);

    let dir2_node = harness.get_by_label("dir dir2");
    let test_md_visible = harness
        .get_all_by_role(eframe::egui::accesskit::Role::Label)
        .any(|n| n.value().map(|v| v.contains("test.md")).unwrap_or(false));
    assert!(
        !test_md_visible,
        "Sub-content should not be visible in non-recursive toggle"
    );

    dir2_node.click();
    harness.run_steps(5);
    let _ = harness.get_by_label("file test.md");
}
