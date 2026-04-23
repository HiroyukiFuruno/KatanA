use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

/// MD037 / no-space-in-emphasis — Spaces inside emphasis markers
pub struct SpacesInEmphasisRule;

impl MarkdownRule for SpacesInEmphasisRule {
    fn id(&self) -> &'static str {
        "MD037"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD037",
            title: "no-space-in-emphasis",
            description: "Spaces inside emphasis markers",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md037.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD037");
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

            let code_spans = Self::extract_inline_code_ranges(line);
            let markers = Self::extract_markers(line);
            self.check_line_markers(
                file_path,
                i,
                line,
                &markers,
                &code_spans,
                &meta,
                &mut diagnostics,
            );
        }

        diagnostics
    }
}

impl SpacesInEmphasisRule {
    fn find_closing_backticks(rest: &str, target_len: usize) -> Option<usize> {
        let mut close_chars = rest.char_indices().peekable();
        while let Some((rel_idx, c)) = close_chars.next() {
            if c != '`' {
                continue;
            }
            let mut close_len = 1;
            while close_chars.peek().is_some_and(|&(_, p)| p == '`') {
                close_len += 1;
                close_chars.next();
            }
            if close_len == target_len {
                return Some(rel_idx + close_len);
            }
        }
        None
    }

    fn extract_inline_code_ranges(line: &str) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        let mut chars = line.char_indices().peekable();
        while let Some((start_idx, c)) = chars.next() {
            if c != '`' {
                continue;
            }
            let mut len = 1;
            while chars.peek().is_some_and(|&(_, p)| p == '`') {
                len += 1;
                chars.next();
            }
            let search_after = start_idx + len;
            let Some(rest) = line.get(search_after..) else {
                continue;
            };
            if let Some(close_end_rel) = Self::find_closing_backticks(rest, len) {
                let end_idx = search_after + close_end_rel;
                ranges.push(start_idx..end_idx);
                while chars.peek().is_some_and(|&(p, _)| p < end_idx) {
                    chars.next();
                }
            }
        }
        ranges
    }

    #[rustfmt::skip]
    fn extract_markers(line: &str) -> Vec<(usize, usize, char)> {
        let mut markers = Vec::new();
        let mut chars = line.char_indices().peekable();
        while let Some((idx, c)) = chars.next() {
            if c != '*' && c != '_' { continue; }
            let mut count = 1;
            while chars.peek().is_some_and(|&(_, p)| p == c) { count += 1; chars.next(); }
            if count <= 2 { markers.push((idx, count, c)); }
        }
        markers
    }

    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn check_line_markers(
        &self, file_path: &Path, line_idx: usize, line: &str, markers: &[(usize, usize, char)],
        code_spans: &[std::ops::Range<usize>], meta: &OfficialRuleMeta, diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        /* WHY: Find invalid spaces inside emphasis */
        for m in 0..markers.len() {
            let (start_idx, len, _kind) = markers[m];
            if code_spans.iter().any(|r| r.contains(&start_idx)) { continue; }
            let after_marker_idx = start_idx + len;
            if !line[after_marker_idx..].starts_with(' ') { continue; }
            let valid_start = start_idx == 0 || line[..start_idx].ends_with(|c: char| c.is_whitespace() || "([{\"'.!?,;:".contains(c));
            if !valid_start { continue; }

            self.find_matching_close(file_path, line_idx, line, markers, code_spans, m, meta, diagnostics);
        }
    }

    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn find_matching_close(
        &self, file_path: &Path, line_idx: usize, line: &str, markers: &[(usize, usize, char)],
        code_spans: &[std::ops::Range<usize>], m: usize, meta: &OfficialRuleMeta, diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        let (start_idx, len, kind) = markers[m];
        let after_marker_idx = start_idx + len;

        /* WHY: Look for the matching closing marker */
        for &(end_start_idx, end_len, end_kind) in markers.iter().skip(m + 1) {
            if end_kind != kind || end_len != len { continue; }
            if code_spans.iter().any(|r| r.contains(&end_start_idx)) { continue; }
            if !line[..end_start_idx].ends_with(' ') { continue; }

            let inner_text = &line[after_marker_idx..end_start_idx];
            /* WHY: Ensure no backticks in between to avoid crossing code spans */
            if inner_text.contains('`') || inner_text.chars().all(|c| c.is_whitespace()) { break; }

            let trimmed_inner = inner_text.trim();
            let marker_str: String = std::iter::repeat_n(kind, len).collect();
            let replacement = format!("{}{}{}", marker_str, trimmed_inner, marker_str);

            let fix = crate::rules::markdown::types::DiagnosticFix {
                start_line: line_idx + 1, start_column: start_idx + 1,
                end_line: line_idx + 1, end_column: end_start_idx + end_len + 1,
                replacement,
            };

            RuleHelpers::push_diag_with_fix(diagnostics, file_path, line_idx, line, meta, DiagnosticSeverity::Warning, Some(fix));
            break;
        }
    }
}
