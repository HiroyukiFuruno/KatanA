use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: Heading-related markdownlint rule implementations
======================================================= */

/// MD003 / heading-style — Enforce consistent heading style (atx).
pub struct HeadingStyleRule;

impl MarkdownRule for HeadingStyleRule {
    fn id(&self) -> &'static str {
        "MD003"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD003",
            title: "heading-style",
            description: "Heading style should be consistent (atx expected).",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md003.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD003");
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
            /* WHY: Detect setext-style headings (underline with === or ---) */
            if is_setext_underline(trimmed) && i > 0 {
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

/// MD022 / blanks-around-headings — Headings should be surrounded by blank lines.
pub struct BlanksAroundHeadingsRule;

impl MarkdownRule for BlanksAroundHeadingsRule {
    fn id(&self) -> &'static str {
        "MD022"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD022",
            title: "blanks-around-headings",
            description: "Headings should be surrounded by blank lines.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md022.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD022");
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_code_block = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block || !RuleHelpers::is_atx_heading(trimmed) {
                continue;
            }
            let needs_blank_before = i > 0 && !lines[i - 1].trim().is_empty();
            let needs_blank_after = i + 1 < lines.len() && !lines[i + 1].trim().is_empty();
            if needs_blank_before || needs_blank_after {
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

/// MD023 / heading-start-left — Headings must start at the beginning of the line.
pub struct HeadingStartLeftRule;

impl MarkdownRule for HeadingStartLeftRule {
    fn id(&self) -> &'static str {
        "MD023"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD023",
            title: "heading-start-left",
            description: "Headings must start at the beginning of the line.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md023.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD023");
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
            /* WHY: A heading with leading whitespace violates this rule */
            if RuleHelpers::is_atx_heading(trimmed) && line != trimmed {
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

/* WHY: Section: Private helpers
======================================================= */

fn is_setext_underline(trimmed: &str) -> bool {
    if trimmed.len() < 2 {
        return false;
    }
    trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-')
}
