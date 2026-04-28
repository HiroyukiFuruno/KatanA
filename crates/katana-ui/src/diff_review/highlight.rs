use super::{DiffLine, DiffLineKind, TextRange};

pub(super) struct DiffHighlightOps;

impl DiffHighlightOps {
    pub(super) fn apply(lines: &mut [DiffLine]) {
        let mut index = 0;
        while index < lines.len() {
            if lines[index].is_unchanged() {
                index += 1;
                continue;
            }

            let group_start = index;
            while index < lines.len() && !lines[index].is_unchanged() {
                index += 1;
            }
            Self::apply_group(&mut lines[group_start..index]);
        }
    }

    pub(super) fn changed_ranges(before: &str, after: &str) -> (Vec<TextRange>, Vec<TextRange>) {
        changed_text_ranges(before, after)
    }

    fn apply_group(lines: &mut [DiffLine]) {
        let removed_indexes = Self::line_indexes(lines, DiffLineKind::Removed);
        let added_indexes = Self::line_indexes(lines, DiffLineKind::Added);
        let paired_count = removed_indexes.len().min(added_indexes.len());

        for pair_index in 0..paired_count {
            let removed_index = removed_indexes[pair_index];
            let added_index = added_indexes[pair_index];
            let removed_text = lines[removed_index].text.clone();
            let added_text = lines[added_index].text.clone();
            let (removed_range, added_range) = changed_text_ranges(&removed_text, &added_text);
            lines[removed_index].highlight_ranges = removed_range;
            lines[added_index].highlight_ranges = added_range;
        }

        for removed_index in removed_indexes.iter().skip(paired_count) {
            lines[*removed_index].highlight_ranges = full_text_range(&lines[*removed_index].text);
        }
        for added_index in added_indexes.iter().skip(paired_count) {
            lines[*added_index].highlight_ranges = full_text_range(&lines[*added_index].text);
        }
    }

    fn line_indexes(lines: &[DiffLine], kind: DiffLineKind) -> Vec<usize> {
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.kind == kind)
            .map(|(index, _)| index)
            .collect()
    }
}

fn changed_text_ranges(before: &str, after: &str) -> (Vec<TextRange>, Vec<TextRange>) {
    let before_chars = before.chars().collect::<Vec<_>>();
    let after_chars = after.chars().collect::<Vec<_>>();
    let matches = matching_char_indexes(&before_chars, &after_chars);
    (
        unmatched_text_ranges(before_chars.len(), matches.iter().map(|it| it.0)),
        unmatched_text_ranges(after_chars.len(), matches.iter().map(|it| it.1)),
    )
}

fn matching_char_indexes(before: &[char], after: &[char]) -> Vec<(usize, usize)> {
    let table = char_lcs_table(before, after);
    let mut matches = Vec::new();
    let mut before_index = 0;
    let mut after_index = 0;

    while before_index < before.len() && after_index < after.len() {
        if before[before_index] == after[after_index] {
            matches.push((before_index, after_index));
            before_index += 1;
            after_index += 1;
        } else if table[before_index + 1][after_index] >= table[before_index][after_index + 1] {
            before_index += 1;
        } else {
            after_index += 1;
        }
    }

    matches
}

fn char_lcs_table(before: &[char], after: &[char]) -> Vec<Vec<usize>> {
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

fn unmatched_text_ranges(
    text_len: usize,
    matched_indexes: impl Iterator<Item = usize>,
) -> Vec<TextRange> {
    let matched = matched_indexes.collect::<std::collections::HashSet<_>>();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < text_len {
        if matched.contains(&index) {
            index += 1;
            continue;
        }

        let start = index;
        while index < text_len && !matched.contains(&index) {
            index += 1;
        }
        ranges.push(TextRange::new(start, index));
    }

    ranges
}

fn full_text_range(text: &str) -> Vec<TextRange> {
    if text.is_empty() {
        Vec::new()
    } else {
        vec![TextRange::new(0, text.chars().count())]
    }
}
