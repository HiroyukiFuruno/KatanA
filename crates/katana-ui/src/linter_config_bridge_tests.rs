use crate::linter_config_bridge::MarkdownLinterConfigOps;
use katana_core::workspace::Workspace;
use std::sync::Arc;

fn make_state() -> crate::app_state::AppState {
    crate::app_state::AppState::new(
        Default::default(),
        Default::default(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    )
}

fn make_workspace_state(dir: &tempfile::TempDir) -> crate::app_state::AppState {
    let mut state = make_state();
    state
        .config
        .settings
        .settings_mut()
        .linter
        .use_workspace_local_config = true;
    state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));
    state
}

#[test]
fn target_config_path_uses_global_or_workspace_config() {
    let mut state = make_state();
    let global_path = MarkdownLinterConfigOps::target_config_path(&state);
    assert!(global_path.ends_with("KatanA/.markdownlint.json"));

    let dir = tempfile::tempdir().expect("tempdir must be available");
    state
        .config
        .settings
        .settings_mut()
        .linter
        .use_workspace_local_config = true;
    state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));
    assert_eq!(
        MarkdownLinterConfigOps::target_config_path(&state),
        dir.path().join(".markdownlint.json")
    );
}

#[test]
fn target_config_path_uses_workspace_jsonc_when_present() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    std::fs::write(
        dir.path().join(".markdownlint.jsonc"),
        "{\n  \"default\": false\n}\n",
    )
    .expect("workspace jsonc must be writable");
    let state = make_workspace_state(&dir);

    assert_eq!(
        MarkdownLinterConfigOps::target_config_path(&state),
        dir.path().join(".markdownlint.jsonc")
    );
}

#[test]
fn load_options_uses_kml_config_conversion() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"default": false, "MD001": true}"#,
    )
    .expect("workspace json must be writable");
    let state = make_workspace_state(&dir);

    let options = MarkdownLinterConfigOps::load_options(&state);

    assert!(
        options
            .rules
            .get("MD001")
            .expect("MD001 must exist")
            .enabled
    );
    assert!(options.rules.values().any(|rule| !rule.enabled));
}

#[test]
fn effective_config_path_ignores_workspace_config_when_disabled() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let workspace_config = dir.path().join(".markdownlint.json");
    let global_config = dir.path().join("global.json");
    std::fs::write(&workspace_config, r#"{"default": false}"#)
        .expect("workspace json must be writable");
    std::fs::write(&global_config, r#"{"default": true}"#).expect("global json must be writable");

    let selected = MarkdownLinterConfigOps::select_effective_config_path(
        false,
        Some(&workspace_config),
        None,
        &global_config,
    );

    assert_eq!(selected, Some(global_config));
}

#[test]
fn effective_config_path_prefers_workspace_json_when_enabled() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let workspace_config = dir.path().join(".markdownlint.json");
    let global_config = dir.path().join("global.json");
    std::fs::write(&workspace_config, r#"{"default": false}"#)
        .expect("workspace json must be writable");
    std::fs::write(&global_config, r#"{"default": true}"#).expect("global json must be writable");

    let selected = MarkdownLinterConfigOps::select_effective_config_path(
        true,
        Some(&workspace_config),
        None,
        &global_config,
    );

    assert_eq!(selected, Some(workspace_config));
}

#[test]
fn effective_config_path_prefers_workspace_jsonc_when_json_missing() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let workspace_json = dir.path().join(".markdownlint.json");
    let workspace_jsonc = dir.path().join(".markdownlint.jsonc");
    let global_config = dir.path().join("global.json");
    std::fs::write(&workspace_jsonc, r#"{"default": false}"#)
        .expect("workspace jsonc must be writable");
    std::fs::write(&global_config, r#"{"default": true}"#).expect("global json must be writable");

    let selected = MarkdownLinterConfigOps::select_effective_config_path(
        true,
        Some(&workspace_json),
        Some(&workspace_jsonc),
        &global_config,
    );

    assert_eq!(selected, Some(workspace_jsonc));
}

#[test]
fn effective_config_path_falls_back_to_global_then_default() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let workspace_config = dir.path().join(".markdownlint.json");
    let global_config = dir.path().join("global.json");

    assert_eq!(
        MarkdownLinterConfigOps::select_effective_config_path(
            true,
            Some(&workspace_config),
            None,
            &global_config,
        ),
        None
    );

    std::fs::write(&global_config, r#"{"default": false}"#).expect("global json must be writable");
    assert_eq!(
        MarkdownLinterConfigOps::select_effective_config_path(
            true,
            Some(&workspace_config),
            None,
            &global_config,
        ),
        Some(global_config)
    );
}

#[test]
fn load_config_or_katana_default_uses_bundled_config_when_missing() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let config =
        MarkdownLinterConfigOps::load_config_or_katana_default(&dir.path().join("missing"));

    assert_eq!(config.raw["MD013"], false);
    assert_eq!(config.raw["MD048"]["style"], "consistent");
}

#[test]
fn load_config_or_katana_default_uses_bundled_config_when_invalid() {
    let dir = tempfile::tempdir().expect("tempdir must be available");
    let invalid_config = dir.path().join(".markdownlint.json");
    std::fs::write(&invalid_config, "{").expect("invalid config must be writable");

    let config = MarkdownLinterConfigOps::load_config_or_katana_default(&invalid_config);

    assert_eq!(config.raw["MD013"], false);
    assert_eq!(config.raw["MD048"]["style"], "consistent");
}

#[test]
fn katana_namespace_is_not_markdownlint_compatible() {
    let config = katana_markdown_linter::MarkdownLintConfig {
        raw: serde_json::json!({
            "katana": {
                "rule_severity": {
                    "MD013": "ignore"
                }
            }
        }),
    };

    let errors = config.validate_cached_rules();

    assert!(
        errors
            .iter()
            .any(|error| error.kind_code() == "unknown_rule")
    );
}
