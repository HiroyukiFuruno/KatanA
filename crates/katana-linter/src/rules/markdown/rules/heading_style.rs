use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

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
            properties: &[crate::rule_prop_enum!(
                "style",
                "Heading style",
                "consistent",
                &[
                    "consistent",
                    "atx",
                    "atx_closed",
                    "setext",
                    "setext_with_atx",
                    "setext_with_atx_closed"
                ]
            )],
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
                let prev_line = content.lines().nth(i - 1).unwrap().trim();
                if !prev_line.is_empty()
                    && !RuleHelpers::is_fence(prev_line)
                    && !is_setext_underline(prev_line)
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
        }
        diagnostics
    }
}

fn is_setext_underline(trimmed: &str) -> bool {
    if trimmed.len() < 2 {
        return false;
    }
    trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-')
}
