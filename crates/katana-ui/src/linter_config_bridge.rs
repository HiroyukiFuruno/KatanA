use std::path::{Path, PathBuf};

pub(crate) struct MarkdownLinterConfigOps;

impl MarkdownLinterConfigOps {
    pub(crate) fn target_config_path(state: &crate::app_state::AppState) -> PathBuf {
        let use_workspace = state
            .config
            .settings
            .settings()
            .linter
            .use_workspace_local_config;
        let global_json_path = Self::global_config_path();
        let workspace_json_path = Self::workspace_json_path(state);
        let workspace_jsonc_path = Self::workspace_jsonc_path(state);

        if use_workspace {
            if let Some(path) = workspace_json_path.as_ref()
                && path.exists()
            {
                return path.clone();
            }
            if let Some(path) = workspace_jsonc_path.as_ref()
                && path.exists()
            {
                return path.clone();
            }
            workspace_json_path.unwrap_or(global_json_path)
        } else {
            global_json_path
        }
    }

    pub(crate) fn effective_config_path(state: &crate::app_state::AppState) -> Option<PathBuf> {
        Self::select_effective_config_path(
            state
                .config
                .settings
                .settings()
                .linter
                .use_workspace_local_config,
            Self::workspace_json_path(state).as_deref(),
            Self::workspace_jsonc_path(state).as_deref(),
            &Self::global_config_path(),
        )
    }

    #[cfg(test)]
    pub(crate) fn load_options(
        state: &crate::app_state::AppState,
    ) -> katana_markdown_linter::LintOptions {
        let target_path = Self::target_config_path(state);
        katana_markdown_linter::MarkdownLintConfig::load(target_path.as_path())
            .unwrap_or_default()
            .to_lint_options()
    }

    pub(crate) fn load_options_for_path(
        state: &crate::app_state::AppState,
        _path: &Path,
    ) -> katana_markdown_linter::LintOptions {
        Self::load_effective_config(state).to_lint_options()
    }

    pub(crate) fn load_effective_config(
        state: &crate::app_state::AppState,
    ) -> katana_markdown_linter::MarkdownLintConfig {
        Self::effective_config_path(state)
            .and_then(|path| katana_markdown_linter::MarkdownLintConfig::load(&path).ok())
            .unwrap_or_default()
    }

    fn select_effective_config_path(
        use_workspace: bool,
        workspace_json_path: Option<&Path>,
        workspace_jsonc_path: Option<&Path>,
        global_json_path: &Path,
    ) -> Option<PathBuf> {
        if use_workspace {
            if let Some(path) = workspace_json_path
                && path.exists()
            {
                return Some(path.to_path_buf());
            }
            if let Some(path) = workspace_jsonc_path
                && path.exists()
            {
                return Some(path.to_path_buf());
            }
        }

        global_json_path
            .exists()
            .then(|| global_json_path.to_path_buf())
    }

    fn global_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("KatanA")
            .join(".markdownlint.json")
    }

    fn workspace_json_path(state: &crate::app_state::AppState) -> Option<PathBuf> {
        state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.join(".markdownlint.json"))
    }

    fn workspace_jsonc_path(state: &crate::app_state::AppState) -> Option<PathBuf> {
        state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.join(".markdownlint.jsonc"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
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

        let dir = tempfile::tempdir().unwrap();
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
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".markdownlint.jsonc"),
            "{\n  \"default\": false\n}\n",
        )
        .unwrap();
        let state = make_workspace_state(&dir);

        assert_eq!(
            MarkdownLinterConfigOps::target_config_path(&state),
            dir.path().join(".markdownlint.jsonc")
        );
    }

    #[test]
    fn load_options_uses_kml_config_conversion() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".markdownlint.json"),
            r#"{"default": false, "MD001": true}"#,
        )
        .unwrap();
        let state = make_workspace_state(&dir);

        let options = MarkdownLinterConfigOps::load_options(&state);

        assert!(options.rules.get("MD001").unwrap().enabled);
        assert!(options.rules.values().any(|rule| !rule.enabled));
    }

    #[test]
    fn effective_config_path_ignores_workspace_config_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let workspace_config = dir.path().join(".markdownlint.json");
        let global_config = dir.path().join("global.json");
        std::fs::write(&workspace_config, r#"{"default": false}"#).unwrap();
        std::fs::write(&global_config, r#"{"default": true}"#).unwrap();

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
        let dir = tempfile::tempdir().unwrap();
        let workspace_config = dir.path().join(".markdownlint.json");
        let global_config = dir.path().join("global.json");
        std::fs::write(&workspace_config, r#"{"default": false}"#).unwrap();
        std::fs::write(&global_config, r#"{"default": true}"#).unwrap();

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
        let dir = tempfile::tempdir().unwrap();
        let workspace_json = dir.path().join(".markdownlint.json");
        let workspace_jsonc = dir.path().join(".markdownlint.jsonc");
        let global_config = dir.path().join("global.json");
        std::fs::write(&workspace_jsonc, r#"{"default": false}"#).unwrap();
        std::fs::write(&global_config, r#"{"default": true}"#).unwrap();

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
        let dir = tempfile::tempdir().unwrap();
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

        std::fs::write(&global_config, r#"{"default": false}"#).unwrap();
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
}
