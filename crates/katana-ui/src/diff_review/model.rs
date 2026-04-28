use super::{DiffCompactionOps, DiffLine, DiffLineKind, FileDiffModel, InlineDiffRow, SplitDiffRow};
use crate::diff_review::highlight::DiffHighlightOps;
use crate::diff_review::split_model::DiffSplitModelOps;
use crate::diff_review::types::FileDiffStats;

const FIRST_LINE_NUMBER: usize = 1;

pub(crate) struct DiffModelOps;

impl DiffModelOps {
    pub(crate) fn build(before: &str, after: &str) -> FileDiffModel {
        let before_lines = split_diff_lines(before);
        let after_lines = split_diff_lines(after);
        let mut inline_lines = build_inline_lines(&before_lines, &after_lines);
        DiffHighlightOps::apply(&mut inline_lines);
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

fn split_diff_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').collect()
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
    DiffSplitModelOps::build(inline_rows)
}
