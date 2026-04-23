use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/* WHY: Section: Style/emphasis markdownlint rule implementations
======================================================= */

/* WHY: Minimum marker length for bold/italic (e.g. `**x**` = 5 chars, so content must be > 4) */
const MIN_BOLD_LEN: usize = 4;
/* WHY: Minimum char count for a valid horizontal rule (---) */
const MIN_HR_CHARS: usize = 3;

/// MD036 / no-emphasis-as-heading — Emphasis used instead of a heading.
pub struct NoEmphasisAsHeadingRule;

impl MarkdownRule for NoEmphasisAsHeadingRule {
    fn id(&self) -> &'static str {
        "MD036"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD036",
            title: "no-emphasis-as-heading",
            description: "Emphasis used instead of a heading.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md036.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
            properties: &[crate::rule_prop!(
                String,
                "punctuation",
                "Punctuation characters",
                ".,;:!?。，；：！？"
            )],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD036");
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_code_block = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if is_emphasis_heading(trimmed, &lines, i) {
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

/// MD035 / hr-style — Horizontal rule style should be consistent.
pub struct HrStyleRule;

impl MarkdownRule for HrStyleRule {
    fn id(&self) -> &'static str {
        "MD035"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD035",
            title: "hr-style",
            description: "Horizontal rule style.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md035.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
            properties: &[crate::rule_prop!(
                String,
                "style",
                "Horizontal rule style",
                "consistent"
            )],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD035");
        let mut diagnostics = Vec::new();
        let mut first_hr: Option<String> = None;
        let mut in_code_block = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if is_horizontal_rule(trimmed) {
                match &first_hr {
                    None => first_hr = Some(trimmed.to_string()),
                    Some(expected) if trimmed != expected.as_str() => {
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

/* WHY: Section: Private helpers
======================================================= */

fn is_emphasis_heading(trimmed: &str, lines: &[&str], idx: usize) -> bool {
    let is_bold = (trimmed.starts_with("**")
        && trimmed.ends_with("**")
        && trimmed.len() > MIN_BOLD_LEN)
        || (trimmed.starts_with("__") && trimmed.ends_with("__") && trimmed.len() > MIN_BOLD_LEN);
    if !is_bold {
        return false;
    }
    let blank_before = idx == 0 || lines[idx - 1].trim().is_empty();
    let blank_after = idx + 1 >= lines.len() || lines[idx + 1].trim().is_empty();
    blank_before && blank_after
}

fn is_horizontal_rule(trimmed: &str) -> bool {
    if trimmed.len() < MIN_HR_CHARS {
        return false;
    }
    let chars: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.len() < MIN_HR_CHARS {
        return false;
    }
    let first = chars[0];
    (first == '-' || first == '*' || first == '_') && chars.iter().all(|c| *c == first)
}
