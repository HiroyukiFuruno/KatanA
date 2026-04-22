use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticRange, DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta,
    RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: Whitespace and blank-line markdownlint rule implementations
======================================================= */

/// MD012 / no-multiple-blanks — Multiple consecutive blank lines.
pub struct NoMultipleBlanksRule;

impl MarkdownRule for NoMultipleBlanksRule {
    fn id(&self) -> &'static str {
        "MD012"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD012",
            title: "no-multiple-blanks",
            description: "Multiple consecutive blank lines.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md012.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD012");
        let mut diagnostics = Vec::new();
        let mut consecutive_blanks = 0;
        let mut in_code_block = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                consecutive_blanks = 0;
                continue;
            }
            if in_code_block {
                continue;
            }
            if trimmed.is_empty() {
                consecutive_blanks += 1;
                if consecutive_blanks > 1 {
                    RuleHelpers::push_diag(
                        &mut diagnostics,
                        file_path,
                        i,
                        line,
                        &meta,
                        DiagnosticSeverity::Warning,
                    );
                }
            } else {
                consecutive_blanks = 0;
            }
        }
        diagnostics
    }
}

/// MD027 / no-multiple-space-blockquote — Multiple spaces after blockquote symbol.
pub struct NoMultipleSpaceBlockquoteRule;

impl MarkdownRule for NoMultipleSpaceBlockquoteRule {
    fn id(&self) -> &'static str {
        "MD027"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD027",
            title: "no-multiple-space-blockquote",
            description: "Multiple spaces after blockquote symbol.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md027.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD027");
        let mut diagnostics = Vec::new();
        let mut in_code_block = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            /* WHY: Detect ">  " (blockquote followed by 2+ spaces) */
            if trimmed
                .strip_prefix('>')
                .is_some_and(|after| after.starts_with("  "))
            {
                RuleHelpers::push_diag(
                    &mut diagnostics,
                    file_path,
                    i,
                    line,
                    &meta,
                    DiagnosticSeverity::Warning,
                );
            }
        }
        diagnostics
    }
}

/// MD047 / single-trailing-newline — Files should end with a single newline character.
pub struct SingleTrailingNewlineRule;

impl MarkdownRule for SingleTrailingNewlineRule {
    fn id(&self) -> &'static str {
        "MD047"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD047",
            title: "single-trailing-newline",
            description: "Files should end with a single newline character.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md047.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD047");
        if content.is_empty() || content.ends_with('\n') {
            return Vec::new();
        }
        let line_count = content.lines().count();
        vec![MarkdownDiagnostic {
            file: file_path.to_path_buf(),
            severity: DiagnosticSeverity::Warning,
            range: DiagnosticRange {
                start_line: line_count,
                start_column: 1,
                end_line: line_count,
                end_column: 1,
            },
            message: meta.description.to_string(),
            rule_id: meta.code.to_string(),
            official_meta: Some(meta),
        }]
    }
}
