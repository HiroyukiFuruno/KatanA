use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};
// use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;

#[test]
fn test_integration_workspace_tab_persistence() {
    /* WHY: Verify that the open tab state for a workspace is persisted when switching workspaces,
     * ensuring the session can be restored later. */
    let mut harness = setup_harness();
    harness.step();

    let ws1 = fresh_temp_dir("katana_test_ws1");
    let file1 = ws1.join("file1.md");
    std::fs::write(&file1, "# WS1").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(ws1.clone()));
    wait_for_workspace_load(&mut harness);

    let abs_file1 = file1.canonicalize().unwrap_or(file1);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_file1.clone()));
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1
    );

    let ws2 = fresh_temp_dir("katana_test_ws2");

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(ws2.clone()));
    wait_for_workspace_load(&mut harness);

    /* WHY: Calculate the same hash key used by the application to verify the correct entry in the settings store. */
    let mut ws_str = ws1.to_string_lossy().to_string();
    if ws_str.ends_with('/') || ws_str.ends_with('\\') {
        ws_str.pop();
    }
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in ws_str.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let state_key = format!("{:x}", hash);

    let cache_json = harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .load_workspace_state(&state_key);
    assert!(
        cache_json.is_some(),
        "Workspace 1 tab state must be saved to config before switching. Key was: {}",
        state_key
    );

    let json_str = cache_json.unwrap();
    assert!(
        json_str.contains("file1.md"),
        "The saved cache must contain the opened tab's path"
    );

    let _ = std::fs::remove_dir_all(&ws1);
    let _ = std::fs::remove_dir_all(&ws2);
}
