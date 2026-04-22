use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: Blockquote-related markdownlint rule implementations
=======================================================
  Split from whitespace.rs to stay within 200-line limit. */

/// MD028 / no-blanks-blockquote — Blank line inside blockquote.
pub struct NoBlanksBlockquoteRule;

impl MarkdownRule for NoBlanksBlockquoteRule {
    fn id(&self) -> &'static str {
        "MD028"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD028",
            title: "no-blanks-blockquote",
            description: "Blank line inside blockquote.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md028.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD028");
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_code_block = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block || !trimmed.is_empty() {
                continue;
            }
            /* WHY: A blank between two blockquote lines creates separated blockquotes */
            let has_bq_before = i > 0 && lines[i - 1].trim_start().starts_with('>');
            let has_bq_after = i + 1 < lines.len() && lines[i + 1].trim_start().starts_with('>');
            if has_bq_before && has_bq_after {
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
