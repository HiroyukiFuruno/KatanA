use super::markdown_formatting_bridge::MarkdownFormattingBridgeOps;
use katana_core::workspace::Workspace;
use std::sync::Arc;

fn make_workspace_state(dir: &tempfile::TempDir) -> crate::app_state::AppState {
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
fn format_content_uses_effective_markdownlint_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".markdownlint.json"), r#"{"MD047": false}"#).unwrap();
    let state = make_workspace_state(&dir);
    let path = dir.path().join("doc.md");

    let outcome = MarkdownFormattingBridgeOps::format_content(&state, &path, "# Title")
        .expect("format should succeed");

    assert_eq!(outcome.content, "# Title");
    assert_eq!(outcome.applied_fixes, 0);
}

#[test]
fn format_content_respects_disabled_linter_setting() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = make_workspace_state(&dir);
    state.config.settings.settings_mut().linter.enabled = false;
    let path = dir.path().join("doc.md");

    let outcome = MarkdownFormattingBridgeOps::format_content(&state, &path, "# Title")
        .expect("format should succeed");

    assert_eq!(outcome.content, "# Title");
    assert_eq!(outcome.applied_fixes, 0);
}

#[test]
fn format_content_does_not_apply_non_layout_fixes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".markdownlint.json"),
        r#"{"default": false, "MD034": true, "MD047": false}"#,
    )
    .unwrap();
    let state = make_workspace_state(&dir);
    let path = dir.path().join("doc.md");

    let outcome = MarkdownFormattingBridgeOps::format_content(&state, &path, "https://example.com")
        .expect("format should succeed");

    assert_eq!(outcome.content, "https://example.com");
    assert_eq!(outcome.applied_fixes, 0);
}

#[test]
fn format_content_avoids_multibyte_md013_panic() {
    let dir = tempfile::tempdir().unwrap();
    let state = make_workspace_state(&dir);
    let path = dir.path().join("doc.md");
    let content = "- [ ] \u{5bfe}\u{8c61}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3} 0.22.7 \u{306e}\u{5909}\u{66f4} ID \u{3068}\u{30b9}\u{30b3}\u{30fc}\u{30d7}\u{304c}\u{78ba}\u{8a8d}\u{3055}\u{308c}\u{3066}\u{3044}\u{308b}\u{3053}\u{3068}\n";

    let outcome = MarkdownFormattingBridgeOps::format_content(&state, &path, content)
        .expect("format should succeed");

    assert_eq!(outcome.content, content);
}
