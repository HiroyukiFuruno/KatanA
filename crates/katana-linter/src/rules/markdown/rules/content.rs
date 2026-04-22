use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticRange, DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta,
    RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: Content-level markdownlint rule implementations
======================================================= */

/// MD033 / no-inline-html — Inline HTML.
pub struct NoInlineHtmlRule;

impl MarkdownRule for NoInlineHtmlRule {
    fn id(&self) -> &'static str {
        "MD033"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD033",
            title: "no-inline-html",
            description: "Inline HTML.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md033.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD033");
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
            if RuleHelpers::contains_html_tag(line) {
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

/// MD040 / fenced-code-language — Fenced code blocks should have a language specified.
pub struct FencedCodeLanguageRule;

impl MarkdownRule for FencedCodeLanguageRule {
    fn id(&self) -> &'static str {
        "MD040"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD040",
            title: "fenced-code-language",
            description: "Fenced code blocks should have a language specified.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md040.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD040");
        let mut diagnostics = Vec::new();
        let mut in_code_block = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if !RuleHelpers::is_fence(trimmed) {
                continue;
            }
            /* WHY: Opening fence only — check if language is specified */
            if !in_code_block {
                let after_fence = trimmed.trim_start_matches('`').trim_start_matches('~');
                if after_fence.trim().is_empty() {
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
            in_code_block = !in_code_block;
        }
        diagnostics
    }
}

/// MD041 / first-line-heading — First line should be a top-level heading.
pub struct FirstLineHeadingRule;

impl MarkdownRule for FirstLineHeadingRule {
    fn id(&self) -> &'static str {
        "MD041"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD041",
            title: "first-line-heading",
            description: "First line in a file should be a top-level heading.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md041.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD041");
        let first_line = match content.lines().next() {
            Some(line) => line,
            None => return Vec::new(),
        };
        let trimmed = first_line.trim();
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            return Vec::new();
        }
        vec![MarkdownDiagnostic {
            file: file_path.to_path_buf(),
            severity: DiagnosticSeverity::Warning,
            range: DiagnosticRange {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: first_line.len().max(1),
            },
            message: meta.description.to_string(),
            rule_id: meta.code.to_string(),
            official_meta: Some(meta),
            fix_info: None,
        }]
    }
}

/// MD042 / no-empty-links — No empty links.
pub struct NoEmptyLinksRule;

impl MarkdownRule for NoEmptyLinksRule {
    fn id(&self) -> &'static str {
        "MD042"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD042",
            title: "no-empty-links",
            description: "No empty links.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md042.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD042");
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
            if line.contains("]()") || line.contains("](#)") {
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
