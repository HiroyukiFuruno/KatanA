mod types;
pub use types::*;

use crate::rules::markdown::helpers::RuleHelpers;
use std::path::Path;

pub trait MarkdownRule {
    fn id(&self) -> &'static str;
    /// Returns official markdownlint metadata for this rule, if any.
    /// `None` means the rule is hidden (internal-only).
    fn official_meta(&self) -> Option<OfficialRuleMeta>;
    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic>;
}

/* WHY: Section: Official rule implementations
======================================================= */

/// MD001 / heading-increment — Heading levels should only increment by one level at a time.
pub struct HeadingIncrementRule;

impl MarkdownRule for HeadingIncrementRule {
    fn id(&self) -> &'static str {
        "MD001"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD001",
            title: "heading-increment",
            description: "Heading levels should only increment by one level at a time.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md001.md",
            parity: RuleParityStatus::Official,
            is_fixable: false,
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self
            .official_meta()
            .expect("official_meta is always Some for MD001");
        let mut diagnostics = Vec::new();
        let mut last_level = 0;
        let mut in_code_block = false;
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if let Some(level) = RuleHelpers::get_heading_level(line) {
                let current_level = level;
                if last_level > 0 && current_level > last_level + 1 {
                    diagnostics.push(MarkdownDiagnostic {
                        file: file_path.to_path_buf(),
                        severity: DiagnosticSeverity::Warning,
                        range: DiagnosticRange {
                            start_line: line_idx + 1,
                            start_column: 1,
                            end_line: line_idx + 1,
                            end_column: line.len(),
                        },
                        message: format!(
                            "{} [Expected: h{}, Actual: h{}]",
                            meta.description,
                            last_level + 1,
                            current_level
                        ),
                        rule_id: meta.code.to_string(),
                        official_meta: Some(meta.clone()),
                        fix_info: None,
                    });
                }
                last_level = current_level;
            }
        }
        diagnostics
    }
}

/* WHY: Section: Hidden / internal rule implementations
=======================================================
 These rules are not part of official markdownlint contract.
 They are kept for internal use but must NOT appear in user-facing
 Problems Panel output. official_meta() returns None to signal this. */

/// Internal broken-link rule. Hidden from user-facing UI.
pub struct BrokenLinkRule;

impl MarkdownRule for BrokenLinkRule {
    fn id(&self) -> &'static str {
        "md-broken-link"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        /* WHY: Broken link detection has no official markdownlint equivalent rule code.
        This rule is hidden from user-facing diagnostics to avoid false official parity claims. */
        None
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let mut diagnostics = Vec::new();
        /* WHY: Running in workspace context lets us resolve local paths relative to the file. */
        let base_dir = file_path.parent().unwrap_or(Path::new(""));
        for (line_idx, line) in content.lines().enumerate() {
            let mut rest = line;
            let mut offset = 0;
            while let Some(start_idx) = rest.find("](") {
                let actual_start = offset + start_idx;
                rest = &rest[start_idx + 2..];
                offset += start_idx + 2;
                if let Some(end_idx) = rest.find(')') {
                    let link = &rest[..end_idx];
                    let absolute_end = offset + end_idx;
                    RuleHelpers::push_broken_link_violation(
                        &mut diagnostics,
                        file_path,
                        line_idx,
                        actual_start,
                        absolute_end,
                        base_dir,
                        link,
                    );
                    rest = &rest[end_idx + 1..];
                    offset += end_idx + 1;
                }
            }
        }
        diagnostics
    }
}

/* WHY: Section: Legacy re-export for backward compatibility
=======================================================
 Downstream code (katana-ui) references HeadingStructureRule by name.
 Re-export as a public alias so callers compile until migrated to HeadingIncrementRule. */
pub use HeadingIncrementRule as HeadingStructureRule;

#[macro_use]
pub mod macros;
#[rustfmt::skip]
pub mod stubs;
pub use stubs::*;
pub mod stubs_regex;
pub use stubs_regex::*;

pub mod helpers;

/* WHY: Rule implementations live in rules/ subdirectory for clean separation */
pub mod rules;
pub use rules::*;

pub mod eval;
pub use eval::*;
