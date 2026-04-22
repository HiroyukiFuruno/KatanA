use crate::rules::markdown::{
    DiagnosticRange, DiagnosticSeverity, MarkdownDiagnostic, OfficialRuleMeta,
};
use std::path::Path;

/* WHY: Section: Shared helper utilities for markdown rule implementations
=======================================================
  Grouped into struct+impl per coding-rules §1.1 (no public free functions).
  Extracted to stay within 200-line file limits per coding-rules §2.1. */

const MAX_HEADING_LEVEL: usize = 6;

pub struct RuleHelpers;

impl RuleHelpers {
    /// Detect whether a line is a fenced code block delimiter.
    pub fn is_fence(trimmed: &str) -> bool {
        trimmed.starts_with("```") || trimmed.starts_with("~~~")
    }

    /// Detect whether a line is an ATX-style heading.
    pub fn is_atx_heading(trimmed: &str) -> bool {
        if !trimmed.starts_with('#') {
            return false;
        }
        let count = trimmed.chars().take_while(|c| *c == '#').count();
        count <= MAX_HEADING_LEVEL && trimmed[count..].starts_with(' ')
    }

    /// Detect whether a line is a list item (bullet or ordered).
    pub fn is_list_item(trimmed: &str) -> bool {
        Self::get_bullet_char(trimmed).is_some() || Self::get_ordered_number(trimmed).is_some()
    }

    /// Returns the bullet character if the line starts with one.
    pub fn get_bullet_char(trimmed: &str) -> Option<char> {
        let first = trimmed.chars().next()?;
        if (first == '-' || first == '*' || first == '+') && trimmed.get(1..2) == Some(" ") {
            return Some(first);
        }
        None
    }

    /// Returns the ordered list number prefix if present.
    pub fn get_ordered_number(trimmed: &str) -> Option<u32> {
        let dot_pos = trimmed.find(". ")?;
        let prefix = &trimmed[..dot_pos];
        prefix.parse::<u32>().ok()
    }

    /// Push a diagnostic with standard structure.
    pub fn push_diag(
        diagnostics: &mut Vec<MarkdownDiagnostic>,
        file_path: &Path,
        line_idx: usize,
        line: &str,
        meta: &OfficialRuleMeta,
        severity: DiagnosticSeverity,
    ) {
        diagnostics.push(MarkdownDiagnostic {
            file: file_path.to_path_buf(),
            severity,
            range: DiagnosticRange {
                start_line: line_idx + 1,
                start_column: 1,
                end_line: line_idx + 1,
                end_column: line.len().max(1),
            },
            message: meta.description.to_string(),
            rule_id: meta.code.to_string(),
            official_meta: Some(meta.clone()),
        });
    }

    /// Detect inline HTML tags outside code spans.
    pub fn contains_html_tag(line: &str) -> bool {
        let mut rest = line;
        let mut in_code = false;
        while let Some(idx) = rest.find(['`', '<']) {
            let ch = rest.as_bytes()[idx];
            if ch == b'`' {
                in_code = !in_code;
                rest = &rest[idx + 1..];
                continue;
            }
            if in_code {
                rest = &rest[idx + 1..];
                continue;
            }
            let after = &rest[idx + 1..];
            let Some(end) = after.find('>') else {
                rest = &rest[idx + 1..];
                continue;
            };
            if Self::is_likely_html_tag(&after[..end]) {
                return true;
            }
            rest = &after[end + 1..];
        }
        false
    }

    /// Returns ATX heading level (1-6) for a line, or None if not a heading.
    pub fn get_heading_level(line: &str) -> Option<usize> {
        if !line.starts_with('#') {
            return None;
        }
        let count = line.chars().take_while(|c| *c == '#').count();
        if line[count..].starts_with(' ') {
            Some(count)
        } else {
            None
        }
    }

    /// Push a broken-link diagnostic for an internal-only rule.
    pub fn push_broken_link_violation(
        diagnostics: &mut Vec<MarkdownDiagnostic>,
        file_path: &Path,
        line_idx: usize,
        actual_start: usize,
        absolute_end: usize,
        base_dir: &Path,
        link: &str,
    ) {
        if link.starts_with("http") || link.starts_with('#') {
            return;
        }
        let target_path = base_dir.join(link);
        if target_path.exists() || target_path.with_extension("md").exists() {
            return;
        }
        diagnostics.push(MarkdownDiagnostic {
            file: file_path.to_path_buf(),
            severity: DiagnosticSeverity::Error,
            range: DiagnosticRange {
                start_line: line_idx + 1,
                start_column: actual_start + 1,
                end_line: line_idx + 1,
                end_column: absolute_end + 2,
            },
            message: format!("Broken local link: {}", link),
            rule_id: "md-broken-link".to_string(),
            official_meta: None,
        });
    }

    fn is_likely_html_tag(content: &str) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }
        let tag_name = trimmed
            .trim_start_matches('/')
            .split(|c: char| c.is_whitespace() || c == '/')
            .next()
            .unwrap_or("");
        matches!(
            tag_name.to_lowercase().as_str(),
            "br" | "hr"
                | "div"
                | "span"
                | "p"
                | "b"
                | "i"
                | "em"
                | "strong"
                | "a"
                | "img"
                | "table"
                | "tr"
                | "td"
                | "th"
                | "ul"
                | "ol"
                | "li"
                | "pre"
                | "code"
                | "blockquote"
                | "details"
                | "summary"
        )
    }
}
