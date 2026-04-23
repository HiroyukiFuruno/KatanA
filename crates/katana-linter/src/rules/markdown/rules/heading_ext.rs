use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: Heading-related markdownlint rules (part 2)
=======================================================
  Split from rules_heading.rs to stay within 200-line limit. */

/// MD025 / single-title/single-h1 — Only one top-level heading in the document.
pub struct SingleH1Rule;

impl MarkdownRule for SingleH1Rule {
    fn id(&self) -> &'static str {
        "MD025"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD025",
            title: "single-title",
            description: "Multiple top-level headings in the same document.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md025.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
            properties: &[
                crate::rule_prop!(
                    String,
                    "front_matter_title",
                    "RegExp for matching title in front matter",
                    "^\\s*title\\s*[:=]"
                ),
                crate::rule_prop!(Number, "level", "Heading level", "1"),
            ],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD025");
        let mut diagnostics = Vec::new();
        let mut h1_count = 0;
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
            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                h1_count += 1;
                if h1_count > 1 {
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
        }
        diagnostics
    }
}

/// MD026 / no-trailing-punctuation — Trailing punctuation in heading.
pub struct NoTrailingPunctuationRule;

impl MarkdownRule for NoTrailingPunctuationRule {
    fn id(&self) -> &'static str {
        "MD026"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD026",
            title: "no-trailing-punctuation",
            description: "Trailing punctuation in heading.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md026.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
            properties: &[crate::rule_prop!(
                String,
                "punctuation",
                "Punctuation characters",
                ".,;:!。，；：！"
            )],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD026");
        let mut diagnostics = Vec::new();
        let mut in_code_block = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block || !RuleHelpers::is_atx_heading(trimmed) {
                continue;
            }
            let heading_text = trimmed.trim_start_matches('#').trim();
            if heading_text
                .chars()
                .last()
                .is_some_and(|c| ".,;:!".contains(c))
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
