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

        let global_config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("KatanA");
        let global_json_path = global_config_dir.join(".markdownlint.json");
        let workspace_json_path = state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.join(".markdownlint.json"));
        let workspace_jsonc_path = state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.join(".markdownlint.jsonc"));

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
        path: &Path,
    ) -> katana_markdown_linter::LintOptions {
        let target_path = Self::effective_config_path(state, path);
        katana_markdown_linter::MarkdownLintConfig::load(target_path.as_path())
            .unwrap_or_default()
            .to_lint_options()
    }

    fn effective_config_path(state: &crate::app_state::AppState, path: &Path) -> PathBuf {
        if state
            .config
            .settings
            .settings()
            .linter
            .use_workspace_local_config
        {
            return Self::target_config_path(state);
        }

        Self::discover_config_path(state, path).unwrap_or_else(|| Self::target_config_path(state))
    }

    fn discover_config_path(state: &crate::app_state::AppState, path: &Path) -> Option<PathBuf> {
        let workspace_root = state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.as_path());

        let mut current = path.parent()?;
        loop {
            if workspace_root.is_some_and(|root| !current.starts_with(root)) {
                break;
            }
            if let Some(config_path) = Self::config_file_in_dir(current) {
                return Some(config_path);
            }
            if workspace_root.is_some_and(|root| current == root) {
                break;
            }
            current = current.parent()?;
        }

        None
    }

    fn config_file_in_dir(dir: &Path) -> Option<PathBuf> {
        let json_path = dir.join(".markdownlint.json");
        if json_path.exists() {
            return Some(json_path);
        }

        let jsonc_path = dir.join(".markdownlint.jsonc");
        if jsonc_path.exists() {
            return Some(jsonc_path);
        }

        None
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
    fn load_options_for_path_discovers_workspace_config_by_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".markdownlint.json"),
            r#"{"default": false}"#,
        )
        .unwrap();
        let nested = dir.path().join("docs");
        std::fs::create_dir_all(&nested).unwrap();
        let mut state = make_state();
        state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));

        let options =
            MarkdownLinterConfigOps::load_options_for_path(&state, &nested.join("doc.md"));

        assert!(options.rules.values().all(|rule| !rule.enabled));
    }

    #[test]
    fn load_options_for_path_discovers_jsonc_config_by_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".markdownlint.jsonc"),
            "{\n  // keep only MD001 enabled\n  \"default\": false,\n  \"MD001\": true,\n}\n",
        )
        .unwrap();
        let mut state = make_state();
        state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));

        let options =
            MarkdownLinterConfigOps::load_options_for_path(&state, &dir.path().join("doc.md"));

        assert!(options.rules.get("MD001").unwrap().enabled);
        assert!(
            options
                .rules
                .iter()
                .filter(|(rule_id, _)| rule_id.as_str() != "MD001")
                .all(|(_, rule)| !rule.enabled)
        );
    }

    #[test]
    fn discover_config_path_stops_at_workspace_root_without_config() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("docs");
        std::fs::create_dir_all(&nested).unwrap();
        let mut state = make_state();
        state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));

        let config_path =
            MarkdownLinterConfigOps::discover_config_path(&state, &nested.join("doc.md"));

        assert!(config_path.is_none());
    }

    #[test]
    fn discover_config_path_does_not_walk_outside_workspace() {
        let workspace_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        let mut state = make_state();
        state.workspace.data = Some(Workspace::new(workspace_dir.path(), Vec::new()));

        let config_path = MarkdownLinterConfigOps::discover_config_path(
            &state,
            &outside_dir.path().join("doc.md"),
        );

        assert!(config_path.is_none());
    }
}
