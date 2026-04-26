use std::path::{Path, PathBuf};

use crate::app_state::StatusType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownFormatOutcome {
    pub(crate) content: String,
    pub(crate) applied_fixes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownFormatFailure {
    pub(crate) path: PathBuf,
    pub(crate) reason: String,
}

impl MarkdownFormatFailure {
    pub(crate) fn new(path: &Path, reason: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct MarkdownFormattingSummary {
    pub(crate) changed_files: usize,
    pub(crate) unchanged_files: usize,
    pub(crate) failures: Vec<MarkdownFormatFailure>,
}

impl MarkdownFormattingSummary {
    pub(crate) fn status_type(&self) -> StatusType {
        if self.failures.is_empty() {
            StatusType::Success
        } else {
            StatusType::Warning
        }
    }

    pub(crate) fn status_message(&self) -> String {
        let status = &crate::i18n::I18nOps::get().status;
        let failed = self.failures.len();
        if failed == 0 {
            let changed = self.changed_files.to_string();
            let unchanged = self.unchanged_files.to_string();
            return crate::i18n::I18nOps::tf(
                &status.format_markdown_success,
                &[
                    ("changed", changed.as_str()),
                    ("unchanged", unchanged.as_str()),
                ],
            );
        }
        let first_failure = &self.failures[0];
        let changed = self.changed_files.to_string();
        let unchanged = self.unchanged_files.to_string();
        let failed = failed.to_string();
        let path = first_failure.path.display().to_string();
        crate::i18n::I18nOps::tf(
            &status.format_markdown_partial_failure,
            &[
                ("changed", changed.as_str()),
                ("unchanged", unchanged.as_str()),
                ("failed", failed.as_str()),
                ("path", path.as_str()),
                ("reason", &first_failure.reason),
            ],
        )
    }
}

pub(crate) struct MarkdownFormattingBridgeOps;

impl MarkdownFormattingBridgeOps {
    pub(crate) fn format_content(
        state: &crate::app_state::AppState,
        path: &Path,
        content: &str,
    ) -> Result<MarkdownFormatOutcome, MarkdownFormatFailure> {
        if !state.config.settings.settings().linter.enabled {
            return Ok(MarkdownFormatOutcome {
                content: content.to_string(),
                applied_fixes: 0,
            });
        }
        let mut options =
            crate::linter_options_bridge::MarkdownLinterOptionsBridgeOps::load_effective_options(
                state, path,
            );
        Self::disable_non_layout_rules(&mut options);
        let fixed = katana_markdown_linter::fix(content, &options)
            .map_err(|err| MarkdownFormatFailure::new(path, err.to_string()))?;
        Ok(MarkdownFormatOutcome {
            content: fixed.content,
            applied_fixes: fixed.applied_fixes,
        })
    }

    fn disable_non_layout_rules(options: &mut katana_markdown_linter::LintOptions) {
        let layout_options = katana_markdown_linter::layout_lint_options();
        for (rule_id, rule_config) in &mut options.rules {
            if layout_options
                .rules
                .get(rule_id)
                .is_none_or(|layout_rule| !layout_rule.enabled)
            {
                rule_config.enabled = false;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
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

        let outcome =
            MarkdownFormattingBridgeOps::format_content(&state, &path, "https://example.com")
                .expect("format should succeed");

        assert_eq!(outcome.content, "https://example.com");
        assert_eq!(outcome.applied_fixes, 0);
    }
}
