use crate::rules::markdown::helpers::RuleHelpers;
use crate::rules::markdown::types::DiagnosticFix;
use crate::rules::markdown::{
    DiagnosticSeverity, MarkdownDiagnostic, MarkdownRule, OfficialRuleMeta, RuleParityStatus,
};
use std::path::Path;

pub struct TableColumnStyleRule;

impl TableColumnStyleRule {
    fn fix_table_row(
        file_path: &Path,
        line_idx: usize,
        line: &str,
        trimmed: &str,
        meta: &OfficialRuleMeta,
        diagnostics: &mut Vec<MarkdownDiagnostic>,
    ) {
        let has_leading = trimmed.starts_with('|');
        let has_trailing = trimmed.trim_end().ends_with('|');

        let segments: Vec<&str> = trimmed.split('|').collect();
        let mut new_cells = Vec::new();
        let cell_count = segments.len();

        for (j, seg) in segments.iter().enumerate() {
            if ((j == 0 && has_leading) || (j == cell_count - 1 && has_trailing))
                && seg.trim().is_empty()
            {
                continue;
            }
            let s = seg.trim();
            if !s.is_empty() {
                new_cells.push(format!(" {} ", s));
            }
        }

        if new_cells.is_empty() {
            return;
        }

        let leading_spaces = line.len() - trimmed.len();
        let mut new_line = String::new();
        new_line.push_str(&line[0..leading_spaces]);

        if has_leading {
            new_line.push('|');
        }

        new_line.push_str(&new_cells.join("|"));

        if has_trailing {
            new_line.push('|');
        }

        let formatted_line = new_line.trim_end().to_string();
        let orig_trim = line.trim_end();

        if formatted_line != orig_trim {
            let fix = DiagnosticFix {
                start_line: line_idx + 1,
                start_column: 1,
                end_line: line_idx + 1,
                end_column: line.len().max(1) + if line.is_empty() { 0 } else { 1 },
                replacement: formatted_line,
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
        }
    }
}

impl MarkdownRule for TableColumnStyleRule {
    fn id(&self) -> &'static str {
        "MD060"
    }

    fn official_meta(&self) -> Option<OfficialRuleMeta> {
        Some(OfficialRuleMeta {
            code: "MD060",
            title: "table-column-style",
            description: "Table column style should be consistent.",
            docs_url: "https://github.com/DavidAnson/markdownlint/blob/main/doc/md060.md",
            parity: RuleParityStatus::Official,
            is_fixable: true,
            properties: &[
                crate::rule_prop_enum!(
                    "style",
                    "Table column style",
                    "any",
                    &["any", "aligned", "compact", "tight"]
                ),
                crate::rule_prop!(
                    Boolean,
                    "aligned_delimiter",
                    "Aligned delimiter columns",
                    "false"
                ),
            ],
        })
    }

    fn evaluate(&self, file_path: &Path, content: &str) -> Vec<MarkdownDiagnostic> {
        let meta = self.official_meta().expect("always Some for MD060");
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut in_code_block = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block || i == 0 {
                continue;
            }

            let prev_line = lines[i - 1].trim();
            /* WHY: A table delimiter row must follow a header row (which contains a pipe). */
            if !prev_line.contains('|') {
                continue;
            }

            /* WHY: Check if current line is a delimiter row. */
            if !trimmed.contains('-')
                || !trimmed
                    .chars()
                    .all(|c| c.is_whitespace() || c == '|' || c == '-' || c == ':')
            {
                continue;
            }

            Self::fix_table_row(file_path, i, line, trimmed, &meta, &mut diagnostics);
        }

        diagnostics
    }
}
