use super::{DiffLine, InlineDiffRow};
use crate::diff_review::types::UnchangedBlock;

const FIRST_LINE_NUMBER: usize = 1;
const UNCHANGED_CONTEXT_LINES: usize = 3;
const COLLAPSE_MIN_LINES: usize = 7;

pub(crate) struct DiffCompactionOps;

impl DiffCompactionOps {
    pub(crate) fn compact_inline_rows(lines: &[DiffLine]) -> Vec<InlineDiffRow> {
        let mut rows = Vec::new();
        let mut index = 0;

        while index < lines.len() {
            if !lines[index].is_unchanged() {
                rows.push(InlineDiffRow::Line(lines[index].clone()));
                index += 1;
                continue;
            }

            let run_start = index;
            while index < lines.len() && lines[index].is_unchanged() {
                index += 1;
            }
            Self::append_unchanged_run(&mut rows, lines, run_start, index);
        }

        rows
    }

    fn append_unchanged_run(
        rows: &mut Vec<InlineDiffRow>,
        lines: &[DiffLine],
        run_start: usize,
        run_end: usize,
    ) {
        let run_len = run_end - run_start;
        if run_len <= COLLAPSE_MIN_LINES {
            for line in &lines[run_start..run_end] {
                rows.push(InlineDiffRow::Line(line.clone()));
            }
            return;
        }

        let keep_front = if run_start == 0 {
            0
        } else {
            UNCHANGED_CONTEXT_LINES
        };
        let keep_back = if run_end == lines.len() {
            0
        } else {
            UNCHANGED_CONTEXT_LINES
        };

        for line in &lines[run_start..run_start + keep_front] {
            rows.push(InlineDiffRow::Line(line.clone()));
        }

        let collapsed_start = run_start + keep_front;
        let collapsed_end = run_end - keep_back;
        if collapsed_start < collapsed_end {
            rows.push(InlineDiffRow::Collapsed(Self::unchanged_block(
                &lines[collapsed_start],
                collapsed_end - collapsed_start,
            )));
        }

        for line in &lines[collapsed_end..run_end] {
            rows.push(InlineDiffRow::Line(line.clone()));
        }
    }

    fn unchanged_block(first_line: &DiffLine, line_count: usize) -> UnchangedBlock {
        UnchangedBlock {
            before_start_line_number: first_line.before_line_number.unwrap_or(FIRST_LINE_NUMBER),
            after_start_line_number: first_line.after_line_number.unwrap_or(FIRST_LINE_NUMBER),
            line_count,
        }
    }
}
