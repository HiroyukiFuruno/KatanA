use crate::integration::harness_utils::{setup_harness, wait_for_workspace_load, wait_for_workspace_tree};
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;
use egui_kittest::kittest::Queryable;

#[test]
fn test_search_modal_include_exclude_options() {
    /* WHY: Verify that the search modal's inclusion and exclusion glob patterns 
     * correctly filter the resulting file list in real-time. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("apple.md"), "# Apple").unwrap();
    std::fs::write(temp_dir.path().join("banana.md"), "# Banana").unwrap();
    std::fs::write(temp_dir.path().join("cherry.md"), "# Cherry").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();
    wait_for_workspace_tree(&mut harness, 1);

    harness.state_mut().app_state_mut().layout.show_search_modal = true;
    harness.step();
    harness.step();

    harness.state_mut().app_state_mut().search.file_search.query = "apple".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 1);

    harness.state_mut().app_state_mut().search.file_search.query = "".to_string();
    harness.state_mut().app_state_mut().search.include_pattern = "banana".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 1);

    harness.state_mut().app_state_mut().search.include_pattern = "".to_string();
    harness.state_mut().app_state_mut().search.exclude_pattern = "banana".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 2);
}

#[test]
fn test_search_sidebar_buttons() {
    /* WHY: Verify that the search $(\text{magnifying glass})$ and filter $(\nabla)$ buttons 
     * are present in the sidebar for easy access. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    let search_title = I18nOps::get().search.modal_title.clone();
    let search_nodes: Vec<_> = harness.query_all_by_label(&search_title).collect();
    assert!(!search_nodes.is_empty(), "Search button must be present in sidebar");

    let filter_nodes: Vec<_> = harness.query_all_by_label("\u{2207}").collect();
    assert!(!filter_nodes.is_empty(), "Filter button (\u{2207}) must be present in sidebar");
}
