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
                        RuleHelpers::push_diag(
                            &mut diagnostics,
                            file_path,
                            i,
                            line,
                            &meta,
                            DiagnosticSeverity::Warning,
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
                    RuleHelpers::push_diag(
                        &mut diagnostics,
                        file_path,
                        i,
                        line,
                        &meta,
                        DiagnosticSeverity::Warning,
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
