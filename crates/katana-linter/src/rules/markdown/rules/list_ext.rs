use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: List-related markdownlint rule implementations (Extended)
======================================================= */

/// MD032 / blanks-around-lists — Lists should be surrounded by blank lines.
pub struct BlanksAroundListsRule;

impl MarkdownRule for BlanksAroundListsRule {
    fn id(&self) -> &'static str {
        "MD032"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD032",
            title: "blanks-around-lists",
            description: "Lists should be surrounded by blank lines.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md032.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD032");
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_code_block = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block || !RuleHelpers::is_list_item(trimmed) {
                continue;
            }
            let prev_is_problem = i > 0
                && !lines[i - 1].trim().is_empty()
                && !RuleHelpers::is_list_item(lines[i - 1].trim_start());
            if prev_is_problem {
                let fix = crate::rules::markdown::types::DiagnosticFix {
                    start_line: i + 1,
                    start_column: 1,
                    end_line: i + 1,
                    end_column: 1,
                    replacement: "\n".to_string(),
                };
                RuleHelpers::push_diag_with_fix(
                    &mut diagnostics,
                    file_path,
                    i,
                    line,
                    &meta,
                    DiagnosticSeverity::Warning,
                    Some(fix),
                );
            }
        }
        diagnostics
    }
}
