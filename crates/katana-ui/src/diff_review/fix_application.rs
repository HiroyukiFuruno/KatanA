use std::path::PathBuf;

use katana_markdown_linter::{
    Fix, FixSafety, LintResult, Range, Severity, rules::markdown::DiagnosticFix,
};

use super::DiffReviewFile;

pub(crate) struct DiagnosticFixApplicationOps;

impl DiagnosticFixApplicationOps {
    pub(crate) fn apply(content: &str, fixes: &[DiagnosticFix]) -> String {
        let results = fixes.iter().map(Self::to_lint_result).collect::<Vec<_>>();
        katana_markdown_linter::fix_with_results(content, &results).content
    }

    pub(crate) fn build_review_file(
        path: PathBuf,
        before: String,
        fixes: &[DiagnosticFix],
    ) -> Option<DiffReviewFile> {
        let after = Self::apply(&before, fixes);
        if before == after {
            return None;
        }
        Some(DiffReviewFile::new(path, before, after))
    }

    fn to_lint_result(fix: &DiagnosticFix) -> LintResult {
        LintResult {
            rule_id: "katana-ui".to_string(),
            rule_name: String::new(),
            message: String::new(),
            message_id: String::new(),
            message_params: std::collections::BTreeMap::new(),
            severity: Severity::Warning,
            line: fix.start_line,
            column: fix.start_column,
            end_line: fix.end_line,
            end_column: fix.end_column,
            fix: Some(Fix {
                range: Range {
                    start_line: fix.start_line,
                    start_column: fix.start_column,
                    end_line: fix.end_line,
                    end_column: fix.end_column,
                },
                replacement: fix.replacement.clone(),
                safety: FixSafety::Safe,
            }),
        }
    }
}
