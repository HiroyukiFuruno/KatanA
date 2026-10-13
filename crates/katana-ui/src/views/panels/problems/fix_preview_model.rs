pub(super) const MAX_DIFF_ROWS: usize = 10;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct FixPreviewRows {
    pub(super) removed: Vec<String>,
    pub(super) added: Vec<String>,
    pub(super) removed_truncated: bool,
    pub(super) added_truncated: bool,
}

pub(super) struct FixPreviewModelOps;

impl FixPreviewModelOps {
    pub(super) fn build(
        fix: &katana_markdown_linter::rules::markdown::DiagnosticFix,
        content: &str,
    ) -> Option<FixPreviewRows> {
        let lines: Vec<&str> = content.lines().collect();
        let original = Self::original_lines(fix, &lines)?.join("\n");
        let (removed, removed_truncated) = Self::limited_lines(&original);
        let (added, added_truncated) = Self::limited_lines(&fix.replacement);

        Some(FixPreviewRows {
            removed,
            added,
            removed_truncated,
            added_truncated,
        })
    }

    fn original_lines<'a>(
        fix: &katana_markdown_linter::rules::markdown::DiagnosticFix,
        lines: &'a [&'a str],
    ) -> Option<&'a [&'a str]> {
        if fix.start_line == 0 || fix.start_line > lines.len() || fix.end_line < fix.start_line {
            return None;
        }

        let start = fix.start_line - 1;
        let end = fix.end_line.min(lines.len());
        Some(&lines[start..end])
    }

    fn limited_lines(text: &str) -> (Vec<String>, bool) {
        let line_count = text.lines().count();
        let lines = text
            .lines()
            .take(MAX_DIFF_ROWS)
            .map(str::to_string)
            .collect();

        (lines, line_count > MAX_DIFF_ROWS)
    }
}
