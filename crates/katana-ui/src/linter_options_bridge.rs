use std::path::Path;

use katana_markdown_linter::LintOptions;
use katana_platform::settings::types::{LinterSettings, RuleSeverity};

pub(crate) struct MarkdownLinterOptionsBridgeOps;

impl MarkdownLinterOptionsBridgeOps {
    pub(crate) fn load_effective_options(
        state: &crate::app_state::AppState,
        path: &Path,
    ) -> LintOptions {
        let mut options =
            crate::linter_config_bridge::MarkdownLinterConfigOps::load_options_for_path(
                state, path,
            );
        Self::apply_katana_settings(&mut options, &state.config.settings.settings().linter);
        options
    }

    fn apply_katana_settings(options: &mut LintOptions, settings: &LinterSettings) {
        for (rule_id, severity) in &settings.rule_severity {
            let rule_config = options.rules.entry(rule_id.clone()).or_default();
            rule_config.enabled = !matches!(severity, RuleSeverity::Ignore);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use katana_core::workspace::Workspace;
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

        let options = MarkdownLinterOptionsBridgeOps::load_effective_options(
            &state,
            &dir.path().join("doc.md"),
        );

        assert!(!options.rules.get("MD001").unwrap().enabled);
    }

    #[test]
    fn katana_warning_enables_rule_in_effective_options() {
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

        let options = MarkdownLinterOptionsBridgeOps::load_effective_options(
            &state,
            &dir.path().join("doc.md"),
        );

        assert!(options.rules.get("MD001").unwrap().enabled);
    }
}
