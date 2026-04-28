use super::{
    DiffCell, DiffLine, DiffLineKind, InlineDiffRow, SplitDiffLine, SplitDiffRow, TextRange,
    UnchangedBlock,
};
use crate::diff_review::highlight::DiffHighlightOps;
use crate::diff_review::split_pairing::DiffSplitPairingOps;

pub(super) struct DiffSplitModelOps;

impl DiffSplitModelOps {
    pub(super) fn build(inline_rows: &[InlineDiffRow]) -> Vec<SplitDiffRow> {
        let mut builder = SplitRowsBuilder::default();
        for row in inline_rows {
            builder.push(row);
        }
        builder.finish()
    }
}

#[derive(Default)]
struct SplitRowsBuilder {
    rows: Vec<SplitDiffRow>,
    removed: Vec<DiffLine>,
    added: Vec<DiffLine>,
    deferred_unchanged: Vec<DiffLine>,
}

impl SplitRowsBuilder {
    fn push(&mut self, row: &InlineDiffRow) {
        match row {
            InlineDiffRow::Line(line) => self.push_line(line),
            InlineDiffRow::Collapsed(block) => self.push_block(block),
        }
    }

    fn finish(mut self) -> Vec<SplitDiffRow> {
        self.flush_changed_rows();
        self.rows
    }

    fn push_line(&mut self, line: &DiffLine) {
        match line.kind {
            DiffLineKind::Unchanged => self.push_unchanged_line(line),
            DiffLineKind::Removed => self.removed.push(line.clone()),
            DiffLineKind::Added => self.added.push(line.clone()),
        }
    }

    fn push_unchanged_line(&mut self, line: &DiffLine) {
        if is_weak_anchor(line) && self.has_pending_changes() {
            self.deferred_unchanged.push(line.clone());
            return;
        }

        self.flush_changed_rows();
        self.rows.push(unchanged_split_row(line));
    }

    fn push_block(&mut self, block: &UnchangedBlock) {
        if is_weak_block(block) && self.has_pending_changes() {
            self.deferred_unchanged.extend(block.lines.clone());
            return;
        }

        self.flush_changed_rows();
        self.rows.push(SplitDiffRow::Collapsed(block.clone()));
    }

    fn flush_changed_rows(&mut self) {
        let pairs = DiffSplitPairingOps::line_pairs(
            &self.removed,
            &self.added,
            self.deferred_unchanged.is_empty(),
        );
        let mut paired_added = vec![false; self.added.len()];

        self.push_removed_and_paired_rows(&pairs, &mut paired_added);
        self.push_unpaired_added_rows(&paired_added);
        self.push_deferred_unchanged_rows();
        self.removed.clear();
        self.added.clear();
        self.deferred_unchanged.clear();
    }

    fn push_removed_and_paired_rows(&mut self, pairs: &[Option<usize>], paired_added: &mut [bool]) {
        for (removed_index, removed_line) in self.removed.iter().enumerate() {
            let Some(added_index) = pairs[removed_index] else {
                self.rows.push(removed_split_row(removed_line));
                continue;
            };

            paired_added[added_index] = true;
            self.rows
                .push(paired_split_row(removed_line, &self.added[added_index]));
        }
    }

    fn push_unpaired_added_rows(&mut self, paired_added: &[bool]) {
        for (added_index, added_line) in self.added.iter().enumerate() {
            if !paired_added[added_index] {
                self.rows.push(added_split_row(added_line));
            }
        }
    }

    fn push_deferred_unchanged_rows(&mut self) {
        for line in &self.deferred_unchanged {
            self.rows.push(unchanged_split_row(line));
        }
    }

    fn has_pending_changes(&self) -> bool {
        !self.removed.is_empty() || !self.added.is_empty()
    }
}

fn is_weak_anchor(line: &DiffLine) -> bool {
    line.text.trim().is_empty()
}

fn is_weak_block(block: &UnchangedBlock) -> bool {
    block.lines.iter().all(is_weak_anchor)
}

fn unchanged_split_row(line: &DiffLine) -> SplitDiffRow {
    SplitDiffRow::Line(SplitDiffLine {
        before: diff_cell(line.before_line_number, line),
        after: diff_cell(line.after_line_number, line),
    })
}

fn removed_split_row(line: &DiffLine) -> SplitDiffRow {
    SplitDiffRow::Line(SplitDiffLine {
        before: diff_cell(line.before_line_number, line),
        after: None,
    })
}

fn added_split_row(line: &DiffLine) -> SplitDiffRow {
    SplitDiffRow::Line(SplitDiffLine {
        before: None,
        after: diff_cell(line.after_line_number, line),
    })
}

fn paired_split_row(removed: &DiffLine, added: &DiffLine) -> SplitDiffRow {
    let (removed_ranges, added_ranges) =
        DiffHighlightOps::changed_ranges(&removed.text, &added.text);
    SplitDiffRow::Line(SplitDiffLine {
        before: diff_cell_with_ranges(removed.before_line_number, removed, removed_ranges),
        after: diff_cell_with_ranges(added.after_line_number, added, added_ranges),
    })
}

fn diff_cell(line_number: Option<usize>, line: &DiffLine) -> Option<DiffCell> {
    line_number.map(|number| DiffCell {
        line_number: number,
        text: line.text.clone(),
        kind: line.kind,
        highlight_ranges: line.highlight_ranges.clone(),
    })
}

fn diff_cell_with_ranges(
    line_number: Option<usize>,
    line: &DiffLine,
    highlight_ranges: Vec<TextRange>,
) -> Option<DiffCell> {
    line_number.map(|number| DiffCell {
        line_number: number,
        text: line.text.clone(),
        kind: line.kind,
        highlight_ranges,
    })
}
