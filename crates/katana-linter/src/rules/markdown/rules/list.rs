use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: List-related markdownlint rule implementations
======================================================= */

/// MD004 / ul-style — Unordered list style. Enforces consistent bullet character.
pub struct UlStyleRule;

impl MarkdownRule for UlStyleRule {
    fn id(&self) -> &'static str {
        "MD004"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD004",
            title: "ul-style",
            description: "Unordered list style.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md004.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[crate::rule_prop_enum!(
                "style",
                "List style",
                "consistent",
                &["consistent", "asterisk", "plus", "dash", "sublist"]
            )],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD004");
        let mut diagnostics = Vec::new();
        let mut first_bullet: Option<char> = None;
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
            if let Some(bullet) = RuleHelpers::get_bullet_char(trimmed) {
                match first_bullet {
                    None => first_bullet = Some(bullet),
                    Some(expected) if bullet != expected => {
                        let bullet_pos = line.find(bullet).unwrap();
                        let fix = crate::rules::markdown::types::DiagnosticFix {
                            start_line: i + 1,
                            start_column: bullet_pos + 1,
                            end_line: i + 1,
                            end_column: bullet_pos + 2,
                            replacement: expected.to_string(),
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
                    _ => {}
                }
            }
        }
        diagnostics
    }
}

/// MD029 / ol-prefix — Ordered list item prefix.
pub struct OlPrefixRule;

impl MarkdownRule for OlPrefixRule {
    fn id(&self) -> &'static str {
        "MD029"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD029",
            title: "ol-prefix",
            description: "Ordered list item prefix.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md029.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[crate::rule_prop_enum!(
                "style",
                "List style",
                "one_or_ordered",
                &["one", "ordered", "one_or_ordered", "zero"]
            )],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD029");
        let mut diagnostics = Vec::new();
        let mut expected_number: u32 = 1;
        let mut in_code_block = false;
        let mut in_list = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if let Some(num) = RuleHelpers::get_ordered_number(trimmed) {
                if !in_list {
                    in_list = true;
                    expected_number = 1;
                }
                if num != expected_number {
                    let dot_pos = line.find(". ").unwrap();
                    let start_col = line.find(|c: char| c.is_ascii_digit()).unwrap();
                    let fix = crate::rules::markdown::types::DiagnosticFix {
                        start_line: i + 1,
                        start_column: start_col + 1,
                        end_line: i + 1,
                        end_column: dot_pos + 1,
                        replacement: expected_number.to_string(),
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
                expected_number += 1;
            } else if !trimmed.is_empty() {
                in_list = false;
            }
        }
        diagnostics
    }
}
