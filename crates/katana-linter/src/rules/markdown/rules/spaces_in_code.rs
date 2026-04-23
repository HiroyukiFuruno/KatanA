use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/// MD038 / no-space-in-code — Spaces inside code span elements
pub struct NoSpaceInCodeRule;

impl MarkdownRule for NoSpaceInCodeRule {
    fn id(&self) -> &'static str {
        "MD038"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD038",
            title: "no-space-in-code",
            description: "Spaces inside code span elements",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md038.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD038");
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

            self.check_line(file_path, i, line, &meta, &mut diagnostics);
        }

        diagnostics
    }
}

impl NoSpaceInCodeRule {
    fn check_line(
        &self,
        file_path: &Path,
        line_idx: usize,
        line: &str,
        meta: &OfficialRuleMeta,
        diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        /* WHY: Simple parser for code spans: */
        let mut chars = line.char_indices().peekable();
        let mut current_span_start: Option<usize> = None;
        let mut backtick_count = 0;

        while let Some((idx, c)) = chars.next() {
            if c != '`' {
                continue;
            }

            let mut count = 1;
            while let Some(&(_next_idx, next_c)) = chars.peek() {
                if next_c != '`' {
                    break;
                }
                count += 1;
                chars.next();
            }

            if let Some(start_idx) = current_span_start {
                if count == backtick_count {
                    self.verify_span(
                        file_path,
                        line_idx,
                        line,
                        start_idx,
                        idx,
                        count,
                        meta,
                        diagnostics,
                    );
                    current_span_start = None;
                    backtick_count = 0;
                }
            } else {
                /* WHY: Opening the span */
                current_span_start = Some(idx);
                backtick_count = count;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn verify_span(
        &self,
        file_path: &Path,
        line_idx: usize,
        line: &str,
        start_idx: usize,
        end_idx: usize,
        count: usize,
        meta: &OfficialRuleMeta,
        diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        /* WHY: Closing the span */
        let inner_start = start_idx + count;
        if inner_start >= end_idx {
            return;
        }

        let inner_text = &line[inner_start..end_idx];

        /* WHY: 1. Pure spaces are allowed */
        if inner_text.chars().all(|c| c == ' ') {
            return;
        }

        let has_leading = inner_text.starts_with(' ');
        let has_trailing = inner_text.ends_with(' ');

        /* WHY: If no spaces at the edges, it's valid */
        if !has_leading && !has_trailing {
            return;
        }

        /* WHY: 2. Exactly one space on BOTH sides is allowed */
        let has_exact_one_leading = has_leading && !inner_text.starts_with("  ");
        let has_exact_one_trailing = has_trailing && !inner_text.ends_with("  ");

        if has_exact_one_leading && has_exact_one_trailing {
            return;
        }

        /* WHY: Space inside code span detected */
        let trimmed_inner = inner_text.trim();
        let needs_padding = trimmed_inner.starts_with('`') || trimmed_inner.ends_with('`');

        let replacement = if needs_padding {
            format!("{0} {1} {0}", "`".repeat(count), trimmed_inner)
        } else {
            format!("{0}{1}{0}", "`".repeat(count), trimmed_inner)
        };

        let fix = Some(crate::rules::markdown::types::DiagnosticFix {
            start_line: line_idx + 1,
            start_column: start_idx + 1,
            end_line: line_idx + 1,
            end_column: end_idx + count + 1,
            replacement,
        });

        RuleHelpers::push_diag_with_fix(
            diagnostics,
            file_path,
            line_idx,
            line,
            meta,
            DiagnosticSeverity::Warning,
            fix,
        );
    }
}
