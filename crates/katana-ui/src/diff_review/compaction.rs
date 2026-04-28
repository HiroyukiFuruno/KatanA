use super::{DiffLine, InlineDiffRow};
use crate::diff_review::types::UnchangedBlock;

const FIRST_LINE_NUMBER: usize = 1;

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
        rows.push(InlineDiffRow::Collapsed(Self::unchanged_block(
            &lines[run_start..run_end],
        )));
    }

    fn unchanged_block(lines: &[DiffLine]) -> UnchangedBlock {
        let first_line = lines.first();
        UnchangedBlock {
            before_start_line_number: first_line
                .and_then(|line| line.before_line_number)
                .unwrap_or(FIRST_LINE_NUMBER),
            after_start_line_number: first_line
                .and_then(|line| line.after_line_number)
                .unwrap_or(FIRST_LINE_NUMBER),
            line_count: lines.len(),
            lines: lines.to_vec(),
        }
    }
}
