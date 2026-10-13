use super::WorkspaceSettingsOps;
use katana_core::workspace::Workspace;
use std::sync::Arc;

fn make_state(dir: &tempfile::TempDir) -> crate::app_state::AppState {
    let mut state = crate::app_state::AppState::new(
        Default::default(),
        Default::default(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));
    state
}

#[test]
fn workspace_config_toggle_does_not_open_advanced_settings() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".markdownlint.json"), "{}").unwrap();
    let mut state = make_state(&dir);
    let is_advanced_open = false;

    WorkspaceSettingsOps::save_workspace_config_toggle(&mut state, true);

    assert!(
        state
            .config
            .settings
            .settings()
            .linter
            .use_workspace_local_config
    );
    assert!(!is_advanced_open);
}

#[test]
fn existing_workspace_config_prefers_workspace_file() {
    let dir = tempfile::tempdir().unwrap();
    let workspace_config = dir.path().join(".markdownlint.json");
    std::fs::write(&workspace_config, "{}").unwrap();
    let state = make_state(&dir);

    assert_eq!(
        WorkspaceSettingsOps::existing_workspace_config_path(&state),
        Some(workspace_config)
    );
}

#[test]
fn existing_workspace_config_accepts_jsonc_file() {
    let dir = tempfile::tempdir().unwrap();
    let workspace_config = dir.path().join(".markdownlint.jsonc");
    std::fs::write(&workspace_config, "{}").unwrap();
    let state = make_state(&dir);

    assert_eq!(
        WorkspaceSettingsOps::existing_workspace_config_path(&state),
        Some(workspace_config)
    );
}
