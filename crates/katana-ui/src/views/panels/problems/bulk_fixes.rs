use std::path::Path;

pub(crate) struct ProblemBulkFixOps;

impl ProblemBulkFixOps {
    pub(crate) fn file_fixes(
        diagnostics: &[katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
    ) -> Vec<katana_markdown_linter::rules::markdown::DiagnosticFix> {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.official_meta.is_some())
            .filter_map(|diagnostic| diagnostic.fix_info.clone())
            .collect()
    }

    pub(crate) fn workspace_batches(
        problems: &std::collections::BTreeMap<
            std::path::PathBuf,
            Vec<katana_markdown_linter::rules::markdown::MarkdownDiagnostic>,
        >,
        diagnostics_state: &crate::app_state::DiagnosticsState,
    ) -> Vec<crate::app_action::LintFixBatch> {
        problems
            .iter()
            .filter_map(|(path, diagnostics)| {
                Self::file_batch(
                    path,
                    diagnostics,
                    diagnostics_state.content_snapshot(path.as_path()),
                )
            })
            .collect()
    }

    pub(crate) fn file_batch(
        path: &Path,
        diagnostics: &[katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
        source: Option<&str>,
    ) -> Option<crate::app_action::LintFixBatch> {
        let fixes = Self::file_fixes(diagnostics);
        if fixes.is_empty() {
            return None;
        }
        Some(crate::app_action::LintFixBatch {
            path: path.to_path_buf(),
            fixes,
            source: source.map(str::to_string),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ProblemBulkFixOps;
    use katana_markdown_linter::rules::markdown::{
        DiagnosticFix, DiagnosticRange, DiagnosticSeverity, MarkdownDiagnostic, OfficialRuleMeta,
        RuleParityStatus,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn fix(replacement: &str) -> DiagnosticFix {
        DiagnosticFix {
            start_line: 1,
            start_column: 0,
            end_line: 1,
            end_column: 1,
            replacement: replacement.to_string(),
        }
    }

    fn diagnostic(rule_id: &'static str, fix_info: Option<DiagnosticFix>) -> MarkdownDiagnostic {
        MarkdownDiagnostic {
            file: PathBuf::from("/tmp/test.md"),
            rule_id: rule_id.to_string(),
            severity: DiagnosticSeverity::Warning,
            message: "message".to_string(),
            range: DiagnosticRange {
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 1,
            },
            fix_info,
            official_meta: Some(OfficialRuleMeta {
                code: rule_id,
                title: "title",
                description: "description",
                docs_url: "",
                parity: RuleParityStatus::Official,
                is_fixable: true,
                properties: &[],
            }),
        }
    }

    #[test]
    fn file_fixes_collects_only_visible_fixable_diagnostics() {
        let mut hidden = diagnostic("MD999", Some(fix("hidden")));
        hidden.official_meta = None;
        let diagnostics = vec![
            diagnostic("MD001", Some(fix("one"))),
            diagnostic("MD002", None),
            hidden,
        ];

        let fixes = ProblemBulkFixOps::file_fixes(&diagnostics);

        assert_eq!(fixes.len(), 1);
        assert_eq!(fixes[0].replacement, "one");
    }

    #[test]
    fn workspace_batches_preserves_file_boundaries() {
        let first = PathBuf::from("/tmp/first.md");
        let second = PathBuf::from("/tmp/second.md");
        let third = PathBuf::from("/tmp/third.md");
        let mut problems = BTreeMap::new();
        problems.insert(first.clone(), vec![diagnostic("MD001", Some(fix("first")))]);
        problems.insert(second.clone(), vec![diagnostic("MD002", None)]);
        problems.insert(third.clone(), vec![diagnostic("MD003", Some(fix("third")))]);

        let mut state = crate::app_state::DiagnosticsState::new();
        state.update_diagnostics_for_content(first.clone(), "first source", Vec::new());
        state.update_diagnostics_for_content(third.clone(), "third source", Vec::new());

        let batches = ProblemBulkFixOps::workspace_batches(&problems, &state);

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].path, first);
        assert_eq!(batches[0].fixes[0].replacement, "first");
        assert_eq!(batches[0].source.as_deref(), Some("first source"));
        assert_eq!(batches[1].path, third);
        assert_eq!(batches[1].fixes[0].replacement, "third");
        assert_eq!(batches[1].source.as_deref(), Some("third source"));
    }
}
