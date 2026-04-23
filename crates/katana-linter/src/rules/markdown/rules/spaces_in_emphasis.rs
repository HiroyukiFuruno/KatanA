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

            let markers = Self::extract_markers(line);
            self.check_line_markers(file_path, i, line, &markers, &meta, &mut diagnostics);
        }

        diagnostics
    }
}

impl SpacesInEmphasisRule {
    fn extract_markers(line: &str) -> Vec<(usize, usize, char)> {
        let mut markers = Vec::new();
        let mut chars = line.char_indices().peekable();
        while let Some((idx, c)) = chars.next() {
            if c != '*' && c != '_' {
                continue;
            }
            let mut count = 1;
            while let Some(&(_next_idx, next_c)) = chars.peek() {
                if next_c != c {
                    break;
                }
                count += 1;
                chars.next();
            }
            if count <= 2 {
                markers.push((idx, count, c));
            }
        }
        markers
    }

    fn check_line_markers(
        &self,
        file_path: &Path,
        line_idx: usize,
        line: &str,
        markers: &[(usize, usize, char)],
        meta: &OfficialRuleMeta,
        diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        /* WHY: Find invalid spaces inside emphasis */
        for m in 0..markers.len() {
            let (start_idx, len, _kind) = markers[m];
            let after_marker_idx = start_idx + len;

            /* WHY: Check if the marker is followed by a space */
            if !line[after_marker_idx..].starts_with(' ') {
                continue;
            }

            /* WHY: Check if it's preceded by whitespace or start of line or punctuation */
            let valid_start = start_idx == 0
                || line[..start_idx]
                    .ends_with(|c: char| c.is_whitespace() || "([{\"'.!?,;:".contains(c));
            if !valid_start {
                continue;
            }

            self.find_matching_close(file_path, line_idx, line, markers, m, meta, diagnostics);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn find_matching_close(
        &self,
        file_path: &Path,
        line_idx: usize,
        line: &str,
        markers: &[(usize, usize, char)],
        m: usize,
        meta: &OfficialRuleMeta,
        diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        let (start_idx, len, kind) = markers[m];
        let after_marker_idx = start_idx + len;

        /* WHY: Look for the matching closing marker */
        for &(end_start_idx, end_len, end_kind) in markers.iter().skip(m + 1) {
            if end_kind != kind || end_len != len {
                continue;
            }

            /* WHY: Check if it's preceded by a space */
            if !line[..end_start_idx].ends_with(' ') {
                break;
            }

            let inner_text = &line[after_marker_idx..end_start_idx];
            /* WHY: Ensure no backticks in between to avoid crossing code spans */
            if inner_text.contains('`') || inner_text.chars().all(|c| c.is_whitespace()) {
                break;
            }

            let trimmed_inner = inner_text.trim();
            let marker_str: String = std::iter::repeat_n(kind, len).collect();
            let replacement = format!("{}{}{}", marker_str, trimmed_inner, marker_str);

            let fix = crate::rules::markdown::types::DiagnosticFix {
                start_line: line_idx + 1,
                start_column: start_idx + 1,
                end_line: line_idx + 1,
                end_column: end_start_idx + end_len + 1,
                replacement,
            };

            RuleHelpers::push_diag_with_fix(
                diagnostics,
                file_path,
                line_idx,
                line,
                meta,
                DiagnosticSeverity::Warning,
                Some(fix),
            );
            break;
        }
    }
}
