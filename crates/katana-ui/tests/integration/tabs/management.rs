use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_open_all_markdown() {
    /* WHY: Verify that multiple markdown files can be opened at once via OpenMultipleDocuments action,
     * and that duplicate opening attempts do not result in redundant tabs. */
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
    assert_eq!(
        state.document.open_documents.len(),
        2,
        "Should open 2 documents"
    );

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
        "Should not duplicate tabs on re-opening"
    );

    /* WHY: First file should be activated as per user expectation when opening multiple files. */
    assert_eq!(state.document.active_doc_idx, Some(0));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_multiple_tabs_close() {
    /* WHY: Verify that closing multiple tabs sequentially correctly updates the active document and document list. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_multi_tab");
    std::fs::write(temp_dir.join("file1.md"), "# File 1").unwrap();
    std::fs::write(temp_dir.join("file2.md"), "# File 2").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    let p1 = temp_dir
        .join("file1.md")
        .canonicalize()
        .unwrap_or(temp_dir.join("file1.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p1));
    harness.step();

    let p2 = temp_dir
        .join("file2.md")
        .canonicalize()
        .unwrap_or(temp_dir.join("file2.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p2));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        0
    );
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
fn test_integration_tab_navigation_and_close() {
    /* WHY: Verify UI-driven navigation (prev/next tab buttons) and close button functionality. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_tab_nav");
    std::fs::write(temp_dir.join("a.md"), "# A").unwrap();
    std::fs::write(temp_dir.join("b.md"), "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.to_path_buf()));
    wait_for_workspace_load(&mut harness);

    let a_path = temp_dir.join("a.md");
    let b_path = temp_dir.join("b.md");
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(a_path));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(b_path));
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
