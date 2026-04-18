use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::{AppAction, ViewMode};
// use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_toc_codeonly_mode_scroll_sync() {
    /* WHY: Verify that the TOC correctly highlights the current heading based on the editor's
     * scroll position when in "Code Only" mode, ensuring synchronization without the preview pane. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_scroll");
    let test_file = temp_dir.join("toc_scroll.md");
    let content =
        "# Heading 1\n\n".to_string() + &"\n".repeat(50) + "# Heading 2\n" + &"\n".repeat(300);
    std::fs::write(&test_file, content).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.step();

    harness.state_mut().trigger_action(AppAction::ToggleToc);

    // Wait for TOC to populate
    for _ in 0..100 {
        harness.step();
        if harness.query_all_by_label("Heading 2").count() > 0 {
            break;
        }
    }

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();

    // Scroll down to Heading 2
    harness.state_mut().app_state_mut().scroll.scroll_to_line = Some(60);
    harness.run_steps(10);

    let active_idx = harness.state_mut().app_state_mut().active_toc_index;
    assert_eq!(active_idx, Some(1), "Heading 2 should be active in TOC");
}
