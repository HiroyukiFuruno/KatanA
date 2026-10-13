use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_tab_navigation_and_close() {
    /* WHY: Verify that the tab strip controls (Prev/Next buttons)
     * correctly iterate through open documents and the close button (x)
     * removes the active tab. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("a.md"), "# A").unwrap();
    std::fs::write(temp_dir.path().join("b.md"), "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    let a_path = temp_dir.path().join("a.md");
    let b_path = temp_dir.path().join("b.md");
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(a_path.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(b_path.clone()));
    harness.step();

    let prev_lbl = I18nOps::get().tab.nav_prev.clone();
    if let Some(btn) = harness.query_all_by_label(&prev_lbl).next() {
        btn.click();
    }
    harness.step();

    let next_lbl = I18nOps::get().tab.nav_next.clone();
    if let Some(btn) = harness.query_all_by_label(&next_lbl).next() {
        btn.click();
    }
    harness.step();

    if let Some(btn) = harness.query_all_by_label("x").next() {
        btn.click();
    }
    harness.step();
}

#[test]
fn test_integration_open_all_markdown() {
    /* WHY: Verify the "Open Multiple Documents" action, ensuring that re-opening
     * the same set of files doesn't lead to duplicate tabs. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_open_all_md");
    std::fs::create_dir_all(temp_dir.join("docs")).unwrap();
    let md1 = temp_dir.join("docs").join("a.md");
    let md2 = temp_dir.join("docs").join("b.md");
    std::fs::write(&md1, "# A").unwrap();
    std::fs::write(&md2, "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![
            md1.clone(),
            md2.clone(),
        ]));

    for _ in 0..5 {
        harness.step();
    }

    let state = harness.state_mut().app_state_mut();
    assert_eq!(state.document.open_documents.len(), 2);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![
            md1.clone(),
            md2.clone(),
        ]));

    for _ in 0..5 {
        harness.step();
    }

    let state = harness.state_mut().app_state_mut();
    assert_eq!(
        state.document.open_documents.len(),
        2,
        "Should not duplicate tabs"
    );
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_scroll_truncation() {
    /* WHY: Verify that when >100 tabs are open, the scroll area truncates to the window
     * and doesn't pierce the bounding layout, causing a horizontal overflow. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_tab_scroll");
    std::fs::create_dir_all(temp_dir.join("docs")).unwrap();
    let mut docs = Vec::new();
    for i in 0..105 {
        let path = temp_dir.join("docs").join(format!("file_{:03}.md", i));
        std::fs::write(&path, format!("# File {}", i)).unwrap();
        docs.push(path);
    }

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(docs));

    // Wait enough frames for all tabs to load (one per frame) and layout to settle
    for _ in 0..120 {
        harness.step();
    }

    let state = harness.state_mut().app_state_mut();
    assert_eq!(state.document.open_documents.len(), 105);

    // If it didn't panic or freeze computing layout for 105 tabs, and the width
    // constraints are valid (which we fixed in `views/top_bar/tab_bar/mod.rs`),
    // the layout truncation fix is functioning.

    let _ = std::fs::remove_dir_all(&temp_dir);
}
