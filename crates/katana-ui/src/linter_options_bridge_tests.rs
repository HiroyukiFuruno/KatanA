use super::linter_options_bridge::MarkdownLinterOptionsBridgeOps;
use katana_core::workspace::Workspace;
use katana_platform::settings::types::RuleSeverity;
use std::sync::Arc;

fn make_state(dir: &tempfile::TempDir) -> crate::app_state::AppState {
    let mut state = crate::app_state::AppState::new(
        Default::default(),
        Default::default(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
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
fn katana_ignore_disables_rule_in_effective_options() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".markdownlint.json"), r#"{"MD001": true}"#).unwrap();
    let mut state = make_state(&dir);
    state
        .config
        .settings
        .settings_mut()
        .linter
        .rule_severity
        .insert("MD001".to_string(), RuleSeverity::Ignore);

    let options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));

    assert!(!options.rules.get("MD001").unwrap().enabled);
}

#[test]
fn katana_warning_does_not_reenable_disabled_markdownlint_rule() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"default": false, "MD001": false}"#,
    )
    .unwrap();
    let mut state = make_state(&dir);
    state
        .config
        .settings
        .settings_mut()
        .linter
        .rule_severity
        .insert("MD001".to_string(), RuleSeverity::Warning);

    let options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));

    assert!(!options.rules.get("MD001").unwrap().enabled);
}

#[test]
fn katana_ignore_keeps_rule_properties_for_restore() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"MD013": {"enabled": true, "line_length": 80}}"#,
    )
    .unwrap();
    let mut state = make_state(&dir);
    state
        .config
        .settings
        .settings_mut()
        .linter
        .rule_severity
        .insert("MD013".to_string(), RuleSeverity::Ignore);

    let ignored_options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    assert!(!ignored_options.rules.get("MD013").unwrap().enabled);
    assert_eq!(
        ignored_options
            .rules
            .get("MD013")
            .unwrap()
            .properties
            .get("line_length"),
        Some(&"80".to_string())
    );

    state
        .config
        .settings
        .settings_mut()
        .linter
        .rule_severity
        .insert("MD013".to_string(), RuleSeverity::Warning);
    let restored_options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));

    assert!(restored_options.rules.get("MD013").unwrap().enabled);
}

#[test]
fn unsafe_multibyte_md013_is_disabled_without_losing_properties() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"MD013": {"enabled": true, "line_length": 80}}"#,
    )
    .unwrap();
    let state = make_state(&dir);
    let mut options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    let content = "- [ ] \u{5bfe}\u{8c61}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3} 0.22.7 \u{306e}\u{5909}\u{66f4} ID \u{3068}\u{30b9}\u{30b3}\u{30fc}\u{30d7}\u{304c}\u{78ba}\u{8a8d}\u{3055}\u{308c}\u{3066}\u{3044}\u{308b}\u{3053}\u{3068}";

    MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(&mut options, content);

    let md013 = options.rules.get("MD013").unwrap();
    assert!(!md013.enabled);
    assert_eq!(md013.properties.get("line_length"), Some(&"80".to_string()));
}

#[test]
fn safe_multibyte_md013_boundary_keeps_rule_enabled() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"MD013": {"enabled": true, "line_length": 120}}"#,
    )
    .unwrap();
    let state = make_state(&dir);
    let mut options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    let content = format!("{}{}\n", "a".repeat(78), "\u{3066}\u{3059}\u{3068}");

    MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(&mut options, &content);

    assert!(options.rules.get("MD013").unwrap().enabled);
}

#[test]
fn missing_md013_rule_keeps_options_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".markdownlint.json"), r#"{"MD001": true}"#).unwrap();
    let state = make_state(&dir);
    let mut options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    options.rules.remove("MD013");

    MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(
        &mut options,
        "\u{3066}\u{3059}\u{3068}",
    );

    assert!(!options.rules.contains_key("MD013"));
}

#[test]
fn unsafe_code_block_md013_boundary_disables_rule() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"MD013": {"enabled": true, "code_block_line_length": 10}}"#,
    )
    .unwrap();
    let state = make_state(&dir);
    let mut options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    let content = format!("```\n{}{}\n```\n", "a".repeat(9), "\u{3066}");

    MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(&mut options, &content);

    assert!(!options.rules.get("MD013").unwrap().enabled);
}

#[test]
fn unsafe_heading_md013_boundary_disables_rule() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"MD013": {"enabled": true, "heading_line_length": 10}}"#,
    )
    .unwrap();
    let state = make_state(&dir);
    let mut options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    let content = format!("# {}{}\n", "a".repeat(7), "\u{3066}");

    MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(&mut options, &content);

    assert!(!options.rules.get("MD013").unwrap().enabled);
}

#[test]
fn unsafe_table_md013_boundary_disables_rule() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"MD013": {"enabled": true, "line_length": 10}}"#,
    )
    .unwrap();
    let state = make_state(&dir);
    let mut options =
        MarkdownLinterOptionsBridgeOps::load_effective_options(&state, &dir.path().join("doc.md"));
    let content = format!("| {}{} |\n| --- |\n| value |\n", "a".repeat(7), "\u{3066}");

    MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(&mut options, &content);

    assert!(!options.rules.get("MD013").unwrap().enabled);
}
