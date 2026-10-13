use super::DiffLine;

const SIMILAR_LINE_THRESHOLD: f32 = 0.45;

pub(super) struct DiffSplitPairingOps;

impl DiffSplitPairingOps {
    pub(super) fn line_pairs(
        removed: &[DiffLine],
        added: &[DiffLine],
        allow_index_fallback: bool,
    ) -> Vec<Option<usize>> {
        let mut pairs = similar_line_pairs(removed, added);
        if allow_index_fallback {
            fill_index_pairs(&mut pairs, added);
        }
        pairs
    }
}

fn similar_line_pairs(removed: &[DiffLine], added: &[DiffLine]) -> Vec<Option<usize>> {
    let mut pairs = vec![None; removed.len()];
    let mut used_added = vec![false; added.len()];
    let mut candidates = similar_line_candidates(removed, added);
    candidates.sort_by(|left, right| right.score.total_cmp(&left.score));

    for candidate in candidates {
        if pairs[candidate.removed_index].is_some() || used_added[candidate.added_index] {
            continue;
        }
        pairs[candidate.removed_index] = Some(candidate.added_index);
        used_added[candidate.added_index] = true;
    }

    pairs
}

fn similar_line_candidates(removed: &[DiffLine], added: &[DiffLine]) -> Vec<LinePairCandidate> {
    let mut candidates = Vec::new();
    for (removed_index, removed_line) in removed.iter().enumerate() {
        for (added_index, added_line) in added.iter().enumerate() {
            let score = line_similarity(&removed_line.text, &added_line.text);
            if score >= SIMILAR_LINE_THRESHOLD {
                candidates.push(LinePairCandidate {
                    removed_index,
                    added_index,
                    score,
                });
            }
        }
    }
    candidates
}

fn fill_index_pairs(pairs: &mut [Option<usize>], added: &[DiffLine]) {
    let mut used_added = pairs.iter().flatten().copied().collect::<Vec<_>>();
    for (removed_index, pair) in pairs.iter_mut().enumerate() {
        if pair.is_some() {
            continue;
        }
        let Some(added_index) = next_unused_added_index(removed_index, added, &used_added) else {
            continue;
        };
        *pair = Some(added_index);
        used_added.push(added_index);
    }
}

fn next_unused_added_index(
    start_index: usize,
    added: &[DiffLine],
    used_added: &[usize],
) -> Option<usize> {
    (start_index..added.len())
        .chain(0..start_index.min(added.len()))
        .find(|index| !used_added.contains(index))
}

fn line_similarity(before: &str, after: &str) -> f32 {
    let before_chars = before.trim().chars().collect::<Vec<_>>();
    let after_chars = after.trim().chars().collect::<Vec<_>>();
    if before_chars.is_empty() || after_chars.is_empty() {
        return 0.0;
    }

    let matched = lcs_length(&before_chars, &after_chars) as f32;
    (matched * 2.0) / (before_chars.len() + after_chars.len()) as f32
}

fn lcs_length(before: &[char], after: &[char]) -> usize {
    let mut previous = vec![0; after.len() + 1];
    let mut current = vec![0; after.len() + 1];

    for before_index in (0..before.len()).rev() {
        current[after.len()] = 0;
        for after_index in (0..after.len()).rev() {
            current[after_index] = if before[before_index] == after[after_index] {
                previous[after_index + 1] + 1
            } else {
                previous[after_index].max(current[after_index + 1])
            };
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[0]
}

struct LinePairCandidate {
    removed_index: usize,
    added_index: usize,
    score: f32,
}
