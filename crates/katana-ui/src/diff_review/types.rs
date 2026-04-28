#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffLineKind {
    Unchanged,
    Removed,
    Added,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl TextRange {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffLine {
    pub(crate) kind: DiffLineKind,
    pub(crate) before_line_number: Option<usize>,
    pub(crate) after_line_number: Option<usize>,
    pub(crate) text: String,
    pub(crate) highlight_ranges: Vec<TextRange>,
}

impl DiffLine {
    pub(crate) fn unchanged(
        before_line_number: usize,
        after_line_number: usize,
        text: &str,
    ) -> Self {
        Self {
            kind: DiffLineKind::Unchanged,
            before_line_number: Some(before_line_number),
            after_line_number: Some(after_line_number),
            text: text.to_string(),
            highlight_ranges: Vec::new(),
        }
    }

    pub(crate) fn removed(line_number: usize, text: &str) -> Self {
        Self {
            kind: DiffLineKind::Removed,
            before_line_number: Some(line_number),
            after_line_number: None,
            text: text.to_string(),
            highlight_ranges: Vec::new(),
        }
    }

    pub(crate) fn added(line_number: usize, text: &str) -> Self {
        Self {
            kind: DiffLineKind::Added,
            before_line_number: None,
            after_line_number: Some(line_number),
            text: text.to_string(),
            highlight_ranges: Vec::new(),
        }
    }

    pub(crate) const fn is_unchanged(&self) -> bool {
        matches!(self.kind, DiffLineKind::Unchanged)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffCell {
    pub(crate) line_number: usize,
    pub(crate) text: String,
    pub(crate) kind: DiffLineKind,
    pub(crate) highlight_ranges: Vec<TextRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SplitDiffLine {
    pub(crate) before: Option<DiffCell>,
    pub(crate) after: Option<DiffCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnchangedBlock {
    pub(crate) before_start_line_number: usize,
    pub(crate) after_start_line_number: usize,
    pub(crate) line_count: usize,
    pub(crate) lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InlineDiffRow {
    Line(DiffLine),
    Collapsed(UnchangedBlock),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SplitDiffRow {
    Line(SplitDiffLine),
    Collapsed(UnchangedBlock),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FileDiffStats {
    pub(crate) added_count: usize,
    pub(crate) removed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileDiffModel {
    pub(crate) inline_rows: Vec<InlineDiffRow>,
    pub(crate) split_rows: Vec<SplitDiffRow>,
    pub(crate) stats: FileDiffStats,
}
