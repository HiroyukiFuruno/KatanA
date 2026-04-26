use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
};

use katana_markdown_linter::{
    LintOptions,
    rules::markdown::{
        DiagnosticSeverity, MarkdownDiagnostic, MarkdownLinterOps, OfficialRuleMeta,
    },
};

pub(crate) struct MarkdownLinterBridgeOps;

impl MarkdownLinterBridgeOps {
    pub(crate) fn evaluate_document(
        state: &crate::app_state::AppState,
        path: &Path,
        content: &str,
    ) -> Vec<MarkdownDiagnostic> {
        let linter_settings = &state.config.settings.settings().linter;
        let mut options =
            crate::linter_options_bridge::MarkdownLinterOptionsBridgeOps::load_effective_options(
                state, path,
            );
        crate::linter_options_bridge::MarkdownLinterOptionsBridgeOps::disable_unsafe_multibyte_md013(
            &mut options,
            content,
        );
        let mut severity_map = Self::severity_map(&options);
        for (rule_id, severity) in &linter_settings.rule_severity {
            if options
                .rules
                .get(rule_id)
                .is_none_or(|rule_config| rule_config.enabled)
            {
                severity_map.insert(rule_id.clone(), Self::configured_severity(severity));
            }
        }

        catch_unwind(AssertUnwindSafe(|| {
            MarkdownLinterOps::evaluate_all(
                path,
                content,
                linter_settings.enabled,
                &severity_map,
                &options.rules,
            )
        }))
        .unwrap_or_default()
    }

    pub(crate) fn has_applicable_fix(diag: &MarkdownDiagnostic) -> bool {
        diag.fix_info.is_some()
    }

    fn severity_map(options: &LintOptions) -> HashMap<String, Option<DiagnosticSeverity>> {
        options
            .rules
            .iter()
            .map(|(rule_id, rule_config)| {
                let severity = rule_config
                    .enabled
                    .then(|| Self::default_severity(options.default_severity));
                (rule_id.clone(), severity)
            })
            .collect()
    }

    fn default_severity(severity: katana_markdown_linter::Severity) -> DiagnosticSeverity {
        match severity {
            katana_markdown_linter::Severity::Error => DiagnosticSeverity::Error,
            katana_markdown_linter::Severity::Warning => DiagnosticSeverity::Warning,
            katana_markdown_linter::Severity::Info => DiagnosticSeverity::Info,
        }
    }

    fn configured_severity(
        severity: &katana_platform::settings::types::RuleSeverity,
    ) -> Option<DiagnosticSeverity> {
        match severity {
            katana_platform::settings::types::RuleSeverity::Ignore => None,
            katana_platform::settings::types::RuleSeverity::Warning => {
                Some(DiagnosticSeverity::Warning)
            }
            katana_platform::settings::types::RuleSeverity::Error => {
                Some(DiagnosticSeverity::Error)
            }
        }
    }

    pub(crate) fn diagnostic_message(
        diag: &katana_markdown_linter::rules::markdown::MarkdownDiagnostic,
    ) -> String {
        let locale =
            katana_markdown_linter::resolve_locale_code(&crate::i18n::I18nOps::get_language());
        let result: katana_markdown_linter::LintResult = diag.clone().into();
        katana_markdown_linter::LocalizedDiagnostic::from_result(&result, locale).message
    }

    pub(crate) fn rule_description(meta: &OfficialRuleMeta) -> String {
        Self::rule_description_for_language(meta, &crate::i18n::I18nOps::get_language())
    }

    fn rule_description_for_language(meta: &OfficialRuleMeta, language_code: &str) -> String {
        katana_markdown_linter::localized_rule_description(
            meta.code,
            meta.description,
            language_code,
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use katana_core::workspace::Workspace;
    use katana_markdown_linter::Severity;
    use katana_platform::settings::types::RuleSeverity;
    use std::path::PathBuf;
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

    fn md001_meta() -> OfficialRuleMeta {
        katana_markdown_linter::rules::markdown::eval::MarkdownLinterOps::get_user_configurable_rules()
            .into_iter()
            .find(|rule| rule.id() == "MD001")
            .and_then(|rule| rule.official_meta())
            .unwrap()
    }

    fn diagnostic_with_fix(
        fix_info: Option<katana_markdown_linter::rules::markdown::DiagnosticFix>,
    ) -> MarkdownDiagnostic {
        let mut meta = md001_meta();
        meta.is_fixable = false;
        MarkdownDiagnostic {
            file: PathBuf::from("doc.md"),
            severity: DiagnosticSeverity::Warning,
            range: katana_markdown_linter::rules::markdown::DiagnosticRange {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 1,
            },
            message: "message".to_string(),
            rule_id: meta.code.to_string(),
            official_meta: Some(meta),
            fix_info,
        }
    }

    #[test]
    fn evaluate_document_applies_katana_severity_overrides() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".markdownlint.json"),
            r#"{"default": false, "MD001": true}"#,
        )
        .unwrap();
        let mut state = make_workspace_state(&dir);
        state
            .config
            .settings
            .settings_mut()
            .linter
            .rule_severity
            .insert("MD001".to_string(), RuleSeverity::Error);

        let diagnostics = MarkdownLinterBridgeOps::evaluate_document(
            &state,
            &dir.path().join("doc.md"),
            "# Title\n### Skipped\n",
        );
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.rule_id == "MD001")
            .unwrap();

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert!(!MarkdownLinterBridgeOps::diagnostic_message(diagnostic).is_empty());
    }

    #[test]
    fn evaluate_document_does_not_reenable_markdownlint_disabled_rule() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".markdownlint.json"),
            r#"{"default": false, "MD001": false}"#,
        )
        .unwrap();
        let mut state = make_workspace_state(&dir);
        state
            .config
            .settings
            .settings_mut()
            .linter
            .rule_severity
            .insert("MD001".to_string(), RuleSeverity::Warning);

        let diagnostics = MarkdownLinterBridgeOps::evaluate_document(
            &state,
            &dir.path().join("doc.md"),
            "# Title\n### Skipped\n",
        );

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_id != "MD001")
        );
    }

    #[test]
    fn evaluate_document_contains_linter_panic_boundary() {
        let state = make_state();
        let multibyte_line = "- [ ] \u{5bfe}\u{8c61}\u{30d0}\u{30fc}\u{30b8}\u{30e7}\u{30f3} 0.22.7 \u{306e}\u{5909}\u{66f4} ID \u{3068}\u{30b9}\u{30b3}\u{30fc}\u{30d7}\u{304c}\u{78ba}\u{8a8d}\u{3055}\u{308c}\u{3066}\u{3044}\u{308b}\u{3053}\u{3068}";

        let diagnostics = MarkdownLinterBridgeOps::evaluate_document(
            &state,
            &PathBuf::from("doc.md"),
            &multibyte_line,
        );

        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_id != "MD013")
        );
    }

    #[test]
    fn has_applicable_fix_depends_on_fix_info_not_metadata_flag() {
        let fix = katana_markdown_linter::rules::markdown::DiagnosticFix {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
            replacement: "fixed".to_string(),
        };

        assert!(MarkdownLinterBridgeOps::has_applicable_fix(
            &diagnostic_with_fix(Some(fix))
        ));
        assert!(!MarkdownLinterBridgeOps::has_applicable_fix(
            &diagnostic_with_fix(None)
        ));
    }

    #[test]
    fn severity_helpers_cover_all_supported_values() {
        assert_eq!(
            MarkdownLinterBridgeOps::default_severity(Severity::Error),
            DiagnosticSeverity::Error
        );
        assert_eq!(
            MarkdownLinterBridgeOps::default_severity(Severity::Warning),
            DiagnosticSeverity::Warning
        );
        assert_eq!(
            MarkdownLinterBridgeOps::default_severity(Severity::Info),
            DiagnosticSeverity::Info
        );
        assert_eq!(
            MarkdownLinterBridgeOps::configured_severity(&RuleSeverity::Ignore),
            None
        );
        assert_eq!(
            MarkdownLinterBridgeOps::configured_severity(&RuleSeverity::Warning),
            Some(DiagnosticSeverity::Warning)
        );
        assert_eq!(
            MarkdownLinterBridgeOps::configured_severity(&RuleSeverity::Error),
            Some(DiagnosticSeverity::Error)
        );
    }

    #[test]
    fn rule_description_uses_kml_locale_aware_api() {
        let meta = md001_meta();

        assert_eq!(
            MarkdownLinterBridgeOps::rule_description_for_language(&meta, "en"),
            meta.description
        );
        assert_ne!(
            MarkdownLinterBridgeOps::rule_description_for_language(&meta, "ja"),
            meta.description
        );
    }

    #[test]
    fn rule_description_uses_current_katana_language() {
        let meta = md001_meta();
        let previous_language = crate::i18n::I18nOps::get_language();
        crate::i18n::I18nOps::set_language("en");

        assert_eq!(
            MarkdownLinterBridgeOps::rule_description(&meta),
            meta.description
        );

        crate::i18n::I18nOps::set_language(&previous_language);
    }

    #[test]
    fn unsupported_rule_description_language_uses_kml_fallback() {
        let meta = md001_meta();

        assert_eq!(
            MarkdownLinterBridgeOps::rule_description_for_language(&meta, "unsupported"),
            meta.description
        );
    }
}
