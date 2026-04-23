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
        let mut in_table = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if RuleHelpers::is_fence(trimmed) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            /* WHY: Check if current line is a delimiter row */
            let is_delimiter_row = trimmed.contains('-')
                && trimmed.contains('|')
                && trimmed
                    .chars()
                    .all(|c| c.is_whitespace() || c == '|' || c == '-' || c == ':');

            if in_table {
                /* WHY: We are inside a table. Does this line look like a table row? */
                if trimmed.contains('|') {
                    /* WHY: It's a data row (or another delimiter row, which is weird but okay) */
                    Self::fix_table_row(file_path, i, line, trimmed, &meta, &mut diagnostics);
                } else {
                    /* WHY: Table ended */
                    in_table = false;
                }
                continue;
            }

            if is_delimiter_row {
                in_table = true;
                /* WHY: Check if previous line could be a header row */
                let prev_has_pipe = i > 0 && lines[i - 1].trim().contains('|');
                if prev_has_pipe {
                    /* WHY: Format the header row (i - 1) */
                    Self::fix_table_row(
                        file_path,
                        i - 1,
                        lines[i - 1],
                        lines[i - 1].trim_start(),
                        &meta,
                        &mut diagnostics,
                    );
                }
                /* WHY: Format the delimiter row (i) */
                Self::fix_table_row(file_path, i, line, trimmed, &meta, &mut diagnostics);
            }
        }

        diagnostics
    }
}
