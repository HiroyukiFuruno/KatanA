use crate::integration::harness_utils::{setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::{AppAction, ViewMode};
use katana_ui::i18n::I18nOps;

#[test]
fn test_regression_preview_content_visible_in_preview_only_mode() {
    /* WHY: Regression check: Ensure that in Preview-Only mode, the content scroll area
     * is correctly sized and visible, even when the header is rendered.
     * Previous bug: header consumed 100% height. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("preview_regression.md");
    std::fs::write(&md_path, "# RegressionTestHeading\n\nSome body text.").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    harness.step();

    let no_preview_label = I18nOps::get().preview.no_preview.clone();
    let no_preview_nodes: Vec<_> = harness.query_all_by_label(&no_preview_label).collect();
    assert!(
        no_preview_nodes.is_empty(),
        "Preview pane must NOT show '{no_preview_label}' when a document is open."
    );
}

#[test]
fn test_regression_preview_content_visible_in_split_mode() {
    /* WHY: Regression check: Ensure that in Split mode, the preview pane on the side
     * correctly displays the content and doesn't show the 'no document' placeholder. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("split_regression.md");
    std::fs::write(&md_path, "# SplitRegressionHeading\n\nSome body text.").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.step();

    let no_preview_label = I18nOps::get().preview.no_preview.clone();
    let no_preview_nodes: Vec<_> = harness.query_all_by_label(&no_preview_label).collect();
    assert!(
        no_preview_nodes.is_empty(),
        "Split mode preview pane must NOT show '{no_preview_label}' when a document is open."
    );
}

#[test]
fn test_split_direction_setting_toggles_correctly() {
    /* WHY: Verify the cyclic toggle of split direction (Vertical/Horizontal)
     * via AppAction, ensuring it updates the tab's local state correctly. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("toggle_test.md");
    std::fs::write(&md_path, "# Toggle Test").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
    );

    harness
        .state_mut()
        .trigger_action(AppAction::SetSplitDirection(
            katana_platform::SplitDirection::Vertical,
        ));
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Vertical,
    );
}
