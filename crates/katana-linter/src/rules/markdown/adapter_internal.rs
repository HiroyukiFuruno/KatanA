use std::path::Path;

/* WHY: Section: InternalAdapter — wraps the current built-in rule engine
======================================================= */

/// Adapter that wraps the current built-in rule engine.
pub struct InternalAdapter {
    severity_map: std::collections::HashMap<String, Option<super::DiagnosticSeverity>>,
}

impl InternalAdapter {
    pub fn new(
        severity_map: std::collections::HashMap<String, Option<super::DiagnosticSeverity>>,
    ) -> Self {
        Self { severity_map }
    }

    /// Convert internal `MarkdownDiagnostic` to the adapter's `LintDiagnostic`.
    fn convert(diag: &super::MarkdownDiagnostic) -> super::adapter::LintDiagnostic {
        let (rule_name, fix) = match &diag.official_meta {
            Some(meta) => (
                meta.title.to_string(),
                diag.fix_info.as_ref().map(|f| super::adapter::LintFix {
                    start_line: f.start_line,
                    start_column: f.start_column,
                    end_line: f.end_line,
                    end_column: f.end_column,
                    replacement: f.replacement.clone(),
                }),
            ),
            None => (diag.rule_id.clone(), None),
        };

        super::adapter::LintDiagnostic {
            rule_id: diag.rule_id.clone(),
            rule_name,
            message: diag.message.clone(),
            severity: diag.severity,
            line: diag.range.start_line,
            column: diag.range.start_column,
            end_line: diag.range.end_line,
            end_column: diag.range.end_column,
            fix,
        }
    }

    /// Convert 1-based (line, col) coordinates to a byte offset.
    ///
    /// DiagnosticFix uses 1-indexed lines and columns. This helper
    /// keeps coordinate translation inside the adapter boundary.
    fn byte_offset_1indexed(line_1: usize, col_1: usize, content: &str) -> usize {
        let mut cur_line = 1usize;
        let mut line_start = 0usize;
        for (byte_idx, c) in content.char_indices() {
            if cur_line == line_1 {
                line_start = byte_idx;
                break;
            }
            if c == '\n' {
                cur_line += 1;
            }
        }
        if cur_line < line_1 {
            return content.len();
        }
        let col0 = col_1.saturating_sub(1);
        let mut byte_idx = line_start;
        for (col, (off, c)) in content[line_start..].char_indices().enumerate() {
            if col == col0 || c == '\n' {
                return line_start + off;
            }
            byte_idx = line_start + off + c.len_utf8();
        }
        byte_idx
    }
}

impl super::adapter::MarkdownLintAdapter for InternalAdapter {
    fn lint(
        &self,
        file_path: &Path,
        content: &str,
        _config: &super::config::MarkdownLintConfig,
    ) -> Vec<super::adapter::LintDiagnostic> {
        let diags =
            super::MarkdownLinterOps::evaluate_all(file_path, content, true, &self.severity_map);
        diags.iter().map(Self::convert).collect()
    }

    fn fix_all(
        &self,
        file_path: &Path,
        content: &str,
        _config: &super::config::MarkdownLintConfig,
    ) -> Option<String> {
        let diags =
            super::MarkdownLinterOps::evaluate_all(file_path, content, true, &self.severity_map);

        let mut fixes: Vec<&super::DiagnosticFix> =
            diags.iter().filter_map(|d| d.fix_info.as_ref()).collect();

        if fixes.is_empty() {
            return None;
        }

        /* WHY: Sort descending so later replacements don't shift earlier offsets. */
        fixes.sort_by(|a, b| {
            b.start_line
                .cmp(&a.start_line)
                .then_with(|| b.start_column.cmp(&a.start_column))
        });

        let mut result = content.to_string();
        for fix in fixes {
            let start = Self::byte_offset_1indexed(fix.start_line, fix.start_column, &result);
            let end = Self::byte_offset_1indexed(fix.end_line, fix.end_column, &result);
            if start <= end && end <= result.len() {
                result.replace_range(start..end, &fix.replacement);
            }
        }

        if result == content {
            None
        } else {
            Some(result)
        }
    }
}
