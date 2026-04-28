use super::{
    DiffCell, DiffCompactionOps, DiffLine, DiffLineKind, FileDiffModel, InlineDiffRow,
    SplitDiffLine, SplitDiffRow,
};
use crate::diff_review::types::FileDiffStats;

const FIRST_LINE_NUMBER: usize = 1;

pub(crate) struct DiffModelOps;

impl DiffModelOps {
    pub(crate) fn build(before: &str, after: &str) -> FileDiffModel {
        let before_lines = before.lines().collect::<Vec<_>>();
        let after_lines = after.lines().collect::<Vec<_>>();
        let inline_lines = build_inline_lines(&before_lines, &after_lines);
        let stats = build_stats(&inline_lines);
        let inline_rows = DiffCompactionOps::compact_inline_rows(&inline_lines);
        let split_rows = build_split_rows(&inline_rows);
        FileDiffModel {
            inline_rows,
            split_rows,
            stats,
        }
    }
}

fn build_inline_lines(before: &[&str], after: &[&str]) -> Vec<DiffLine> {
    let table = lcs_table(before, after);
    let mut lines = Vec::new();
    let mut before_index = 0;
    let mut after_index = 0;

    while before_index < before.len() && after_index < after.len() {
        if before[before_index] == after[after_index] {
            lines.push(DiffLine::unchanged(
                before_index + FIRST_LINE_NUMBER,
                after_index + FIRST_LINE_NUMBER,
                before[before_index],
            ));
            before_index += 1;
            after_index += 1;
        } else if table[before_index + 1][after_index] >= table[before_index][after_index + 1] {
            lines.push(DiffLine::removed(
                before_index + FIRST_LINE_NUMBER,
                before[before_index],
            ));
            before_index += 1;
        } else {
            lines.push(DiffLine::added(
                after_index + FIRST_LINE_NUMBER,
                after[after_index],
            ));
            after_index += 1;
        }
    }

    while before_index < before.len() {
        lines.push(DiffLine::removed(
            before_index + FIRST_LINE_NUMBER,
            before[before_index],
        ));
        before_index += 1;
    }
    while after_index < after.len() {
        lines.push(DiffLine::added(
            after_index + FIRST_LINE_NUMBER,
            after[after_index],
        ));
        after_index += 1;
    }

    lines
}

fn lcs_table(before: &[&str], after: &[&str]) -> Vec<Vec<usize>> {
    let mut table = vec![vec![0; after.len() + 1]; before.len() + 1];

    for before_index in (0..before.len()).rev() {
        for after_index in (0..after.len()).rev() {
            table[before_index][after_index] = if before[before_index] == after[after_index] {
                table[before_index + 1][after_index + 1] + 1
            } else {
                table[before_index + 1][after_index].max(table[before_index][after_index + 1])
            };
        }
    }

    table
}

fn build_stats(lines: &[DiffLine]) -> FileDiffStats {
    FileDiffStats {
        added_count: lines
            .iter()
            .filter(|line| line.kind == DiffLineKind::Added)
            .count(),
        removed_count: lines
            .iter()
            .filter(|line| line.kind == DiffLineKind::Removed)
            .count(),
    }
}

fn build_split_rows(inline_rows: &[InlineDiffRow]) -> Vec<SplitDiffRow> {
    let mut rows = Vec::new();
    let mut removed = Vec::new();
    let mut added = Vec::new();

    for row in inline_rows {
        match row {
            InlineDiffRow::Line(line) => match line.kind {
                DiffLineKind::Unchanged => {
                    flush_changed_rows(&mut rows, &mut removed, &mut added);
                    rows.push(SplitDiffRow::Line(SplitDiffLine {
                        before: diff_cell(line.before_line_number, line),
                        after: diff_cell(line.after_line_number, line),
                    }));
                }
                DiffLineKind::Removed => removed.push(line.clone()),
                DiffLineKind::Added => added.push(line.clone()),
            },
            InlineDiffRow::Collapsed(block) => {
                flush_changed_rows(&mut rows, &mut removed, &mut added);
                rows.push(SplitDiffRow::Collapsed(block.clone()));
            }
        }
    }

    flush_changed_rows(&mut rows, &mut removed, &mut added);
    rows
}

fn flush_changed_rows(
    rows: &mut Vec<SplitDiffRow>,
    removed: &mut Vec<DiffLine>,
    added: &mut Vec<DiffLine>,
) {
    let row_count = removed.len().max(added.len());
    for row_index in 0..row_count {
        let before = removed
            .get(row_index)
            .and_then(|line| diff_cell(line.before_line_number, line));
        let after = added
            .get(row_index)
            .and_then(|line| diff_cell(line.after_line_number, line));
        rows.push(SplitDiffRow::Line(SplitDiffLine { before, after }));
    }
    removed.clear();
    added.clear();
}

fn diff_cell(line_number: Option<usize>, line: &DiffLine) -> Option<DiffCell> {
    line_number.map(|number| DiffCell {
        line_number: number,
        text: line.text.clone(),
        kind: line.kind,
    })
}
