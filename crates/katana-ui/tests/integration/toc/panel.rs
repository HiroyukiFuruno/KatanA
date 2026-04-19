use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_toc_panel_display() {
    /* WHY: Verify that the TOC panel is pinned and shows headings when ToggleToc is triggered.
     * trigger_action is used instead of UI button click because egui's data_mut calls
     * in the same frame can interfere with clicked() evaluation in the test harness.
     * The button's accessible label is verified separately to confirm it is rendered correctly. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc");
    let test_file1 = temp_dir.join("toc_test1.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();
    let test_file1 = test_file1.canonicalize().unwrap_or(test_file1);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file1.clone()));
    harness.step();
    harness.step();

    /* WHY: Verify the toggle button is present in the rendered UI with the correct label. */
    let toggle_btn_label = I18nOps::get().action.toggle_toc.clone();
    assert!(
        harness.query_by_label(&toggle_btn_label).is_some(),
        "TOC toggle button must be present in the sidebar"
    );

    harness.state_mut().trigger_action(AppAction::ToggleToc);
    harness.step();
    harness.step();

    let toc_visible = harness.state_mut().app_state_mut().layout.show_toc;
    assert!(toc_visible, "TOC should be pinned after ToggleToc action");

    let headings_count = harness.query_all_by_label("Heading 1").count();
    assert_eq!(
        headings_count, 2,
        "Heading 1 should appear in TOC and preview"
    );
}

#[test]
fn test_integration_toc_panel_truncates_long_headings() {
    /* WHY: Verify that the TOC panel correctly constrains its width even when
     * containing extremely long headings, ensuring usability in small sidebars. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_truncate");
    let test_file = temp_dir.join("toc_truncate.md");
    let long_text = "A".repeat(100);
    std::fs::write(&test_file, format!("# {long_text}")).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));

    for _ in 0..20 {
        harness.step();
    }
    harness.state_mut().trigger_action(AppAction::ToggleToc);
    harness.run_steps(10);

    let toc_title = I18nOps::get().toc.title.clone();
    let panel_node = harness.get_by_label(&toc_title);
    assert!(panel_node.rect().width() <= 550.0);
}
