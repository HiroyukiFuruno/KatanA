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
        Self::load_config_or_katana_default(target_path.as_path()).to_lint_options()
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
            .map(|path| Self::load_config_or_katana_default(&path))
            .unwrap_or_else(crate::linter_default_config::MarkdownLinterDefaultConfigOps::load)
    }

    pub(crate) fn load_config_or_katana_default(
        path: &Path,
    ) -> katana_markdown_linter::MarkdownLintConfig {
        if path.exists() {
            return katana_markdown_linter::MarkdownLintConfig::load(path).unwrap_or_else(|_| {
                crate::linter_default_config::MarkdownLinterDefaultConfigOps::load()
            });
        }
        crate::linter_default_config::MarkdownLinterDefaultConfigOps::load()
    }

    pub(crate) fn select_effective_config_path(
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
