use katana_linter::rules::markdown::DiagnosticFix;

pub(crate) struct LintFixApplication;

impl LintFixApplication {
    pub(crate) fn apply_to_content(content: &str, fixes: &[DiagnosticFix]) -> String {
        let mut buffer = content.to_string();
        let mut sorted_fixes = fixes.to_vec();
        Self::sort_descending(&mut sorted_fixes);
        for fix in &sorted_fixes {
            Self::apply_one(&mut buffer, fix);
        }
        buffer
    }

    fn sort_descending(fixes: &mut [DiagnosticFix]) {
        fixes.sort_by(|left, right| {
            right
                .start_line
                .cmp(&left.start_line)
                .then_with(|| right.start_column.cmp(&left.start_column))
        });
    }

    fn apply_one(buffer: &mut String, fix: &DiagnosticFix) {
        let start = Self::byte_index(buffer, fix.start_line, fix.start_column);
        let end = Self::byte_index(buffer, fix.end_line, fix.end_column);
        let (Some(start), Some(end)) = (start, end) else {
            return;
        };
        if start <= end && end <= buffer.len() {
            buffer.replace_range(start..end, &fix.replacement);
        }
    }

    fn byte_index(buffer: &str, line: usize, column: usize) -> Option<usize> {
        crate::views::panels::editor::types::EditorLogicOps::line_col_to_byte_index(
            buffer,
            line.saturating_sub(1),
            column,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(
        line: usize,
        start_column: usize,
        end_column: usize,
        replacement: &str,
    ) -> DiagnosticFix {
        DiagnosticFix {
            start_line: line,
            start_column,
            end_line: line,
            end_column,
            replacement: replacement.to_string(),
        }
    }

    #[test]
    fn applies_fixes_from_bottom_to_top() {
        let fixes = vec![fix(1, 1, 1, "# "), fix(2, 1, 1, "- ")];
        let fixed = LintFixApplication::apply_to_content("Title\nItem", &fixes);

        assert_eq!(fixed, "# Title\n- Item");
    }
}
