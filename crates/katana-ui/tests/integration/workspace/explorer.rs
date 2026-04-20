use crate::integration::harness_utils::{
    fresh_temp_dir, setup_harness, unique_temp_path, wait_for_workspace_load,
};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

#[test]
fn test_integration_workspace_directory_toggle_non_recursive() {
    /* WHY: Verify that clicking a directory in the explorer tree toggles its expansion state,
     * that expansion is non-recursive by default, and that state is cached correctly. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = unique_temp_path("katana_test_dir_toggle");
    let _ = std::fs::remove_dir_all(&temp_dir);
    let dir2 = temp_dir.join("dir1").join("dir2");
    std::fs::create_dir_all(&dir2).unwrap();
    let test_file = dir2.join("test.md");
    std::fs::write(&test_file, "# Content").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness.step();
    harness.step();

    let dir1_node = harness.get_by_label("dir dir1");

    dir1_node.click();
    harness.step();
    harness.step();

    let dir2_node = harness.get_by_label("dir dir2");

    let test_md_visible = harness
        .get_all_by_role(egui::accesskit::Role::Label)
        .any(|n| n.value().map(|v| v.contains("test.md")).unwrap_or(false));
    assert!(
        !test_md_visible,
        "test.md should NOT be visible (non-recursive expansion)"
    );

    dir2_node.click();
    harness.step();
    harness.step();

    let _ = harness.get_by_label("file test.md");

    let cache_before = harness
        .state_mut()
        .app_state_mut()
        .workspace
        .expanded_directories
        .clone();
    assert!(
        !cache_before.is_empty(),
        "Cache should contain expanded dirs"
    );

    let parent_label = harness.get_by_label("dir dir1");
    parent_label.click();
    harness.step();
    harness.step();

    let parent_label = harness.get_by_label("dir dir1");
    parent_label.click();
    harness.step();
    harness.step();

    let test_md_visible_cached = harness
        .get_all_by_role(egui::accesskit::Role::Label)
        .any(|n| n.value().map(|v| v.contains("test.md")).unwrap_or(false));
    assert!(
        test_md_visible_cached,
        "test.md should be visible after closing and reopening dir1 (cached expansion)"
    );

    /* WHY: The collapse all button has text "-" */
    let collapse_all = harness.get_by_label("-");
    collapse_all.click();
    harness.step();
    harness.step();

    let dir2_present = harness
        .get_all_by_role(egui::accesskit::Role::Label)
        .any(|n| n.value().map(|l| l.contains("dir2")).unwrap_or(false));
    assert!(
        !dir2_present,
        "dir2 should NOT be visible after Collapse All"
    );

    let cache_after = harness
        .state_mut()
        .app_state_mut()
        .workspace
        .expanded_directories
        .clone();
    assert!(
        cache_after.is_empty(),
        "Cache should be EMPTY after Collapse All"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_directory_collapse_bug() {
    /* WHY: Regression test for a bug where collapsing a parent directory didn't properly hide nested child directory entries. */
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
    harness.step();

    let parent_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir parent")
    }))
    .is_ok();
    assert!(parent_visible, "Parent should still be visible");

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
    harness.step();

    let parent_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir parent")
    }))
    .is_ok();
    assert!(parent_visible, "Parent should still be visible");

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
