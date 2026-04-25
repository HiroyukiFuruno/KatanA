#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Unchanged,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffPreviewLine {
    pub kind: DiffLineKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffPreview {
    pub lines: Vec<DiffPreviewLine>,
}

impl DiffPreview {
    pub fn from_contents(old_content: &str, new_content: &str) -> Self {
        let old_lines = Self::content_lines(old_content);
        let new_lines = Self::content_lines(new_content);
        let table = DiffLcsTable::new(&old_lines, &new_lines);
        Self {
            lines: DiffLineBuilder::build(&old_lines, &new_lines, &table),
        }
    }

    pub fn has_changes(&self) -> bool {
        self.lines
            .iter()
            .any(|line| line.kind != DiffLineKind::Unchanged)
    }

    fn content_lines(content: &str) -> Vec<&str> {
        if content.is_empty() {
            Vec::new()
        } else {
            content.split('\n').collect()
        }
    }
}

struct DiffLcsTable {
    cells: Vec<Vec<usize>>,
}

impl DiffLcsTable {
    fn new(old_lines: &[&str], new_lines: &[&str]) -> Self {
        let mut cells = vec![vec![0; new_lines.len() + 1]; old_lines.len() + 1];
        for old_index in (0..old_lines.len()).rev() {
            for new_index in (0..new_lines.len()).rev() {
                cells[old_index][new_index] =
                    Self::next_value(old_lines, new_lines, &cells, old_index, new_index);
            }
        }
        Self { cells }
    }

    fn next_value(
        old_lines: &[&str],
        new_lines: &[&str],
        cells: &[Vec<usize>],
        old_index: usize,
        new_index: usize,
    ) -> usize {
        if old_lines[old_index] == new_lines[new_index] {
            cells[old_index + 1][new_index + 1] + 1
        } else {
            cells[old_index + 1][new_index].max(cells[old_index][new_index + 1])
        }
    }

    fn should_remove(&self, old_index: usize, new_index: usize) -> bool {
        self.cells[old_index + 1][new_index] >= self.cells[old_index][new_index + 1]
    }
}

struct DiffLineBuilder;

impl DiffLineBuilder {
    fn build(old_lines: &[&str], new_lines: &[&str], table: &DiffLcsTable) -> Vec<DiffPreviewLine> {
        let mut lines = Vec::new();
        let (mut old_index, mut new_index) = (0, 0);
        while old_index < old_lines.len() && new_index < new_lines.len() {
            if old_lines[old_index] == new_lines[new_index] {
                Self::push_unchanged(&mut lines, old_lines[old_index], old_index, new_index);
                old_index += 1;
                new_index += 1;
            } else if table.should_remove(old_index, new_index) {
                Self::push_removed(&mut lines, old_lines[old_index], old_index);
                old_index += 1;
            } else {
                Self::push_added(&mut lines, new_lines[new_index], new_index);
                new_index += 1;
            }
        }
        Self::push_remaining(old_lines, new_lines, old_index, new_index, &mut lines);
        lines
    }

    fn push_unchanged(
        lines: &mut Vec<DiffPreviewLine>,
        text: &str,
        old_index: usize,
        new_index: usize,
    ) {
        lines.push(DiffPreviewLine {
            kind: DiffLineKind::Unchanged,
            old_line: Some(old_index + 1),
            new_line: Some(new_index + 1),
            text: text.to_string(),
        });
    }

    fn push_removed(lines: &mut Vec<DiffPreviewLine>, text: &str, old_index: usize) {
        lines.push(DiffPreviewLine {
            kind: DiffLineKind::Removed,
            old_line: Some(old_index + 1),
            new_line: None,
            text: text.to_string(),
        });
    }

    fn push_added(lines: &mut Vec<DiffPreviewLine>, text: &str, new_index: usize) {
        lines.push(DiffPreviewLine {
            kind: DiffLineKind::Added,
            old_line: None,
            new_line: Some(new_index + 1),
            text: text.to_string(),
        });
    }

    fn push_remaining(
        old_lines: &[&str],
        new_lines: &[&str],
        mut old_index: usize,
        mut new_index: usize,
        lines: &mut Vec<DiffPreviewLine>,
    ) {
        while old_index < old_lines.len() {
            Self::push_removed(lines, old_lines[old_index], old_index);
            old_index += 1;
        }
        while new_index < new_lines.len() {
            Self::push_added(lines, new_lines[new_index], new_index);
            new_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_keeps_line_numbers_for_added_and_removed_lines() {
        let preview = DiffPreview::from_contents("a\nb\nc\n", "a\nx\nc\n");

        assert_eq!(preview.lines[1].old_line, Some(2));
        assert_eq!(preview.lines[1].new_line, None);
        assert_eq!(preview.lines[1].kind, DiffLineKind::Removed);
        assert_eq!(preview.lines[2].old_line, None);
        assert_eq!(preview.lines[2].new_line, Some(2));
        assert_eq!(preview.lines[2].kind, DiffLineKind::Added);
    }

    #[test]
    fn preview_reports_no_changes_for_identical_content() {
        let preview = DiffPreview::from_contents("# Title\n\nBody", "# Title\n\nBody");

        assert!(!preview.has_changes());
    }

    #[test]
    fn preview_reports_added_trailing_newline() {
        let preview = DiffPreview::from_contents("# Title", "# Title\n");

        assert!(preview.has_changes());
        assert_eq!(preview.lines[1].kind, DiffLineKind::Added);
        assert_eq!(preview.lines[1].new_line, Some(2));
        assert_eq!(preview.lines[1].text, "");
    }
}
