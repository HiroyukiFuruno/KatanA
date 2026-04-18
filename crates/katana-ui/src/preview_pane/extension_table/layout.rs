use egui_commonmark::Table;

const MIN_FRACTION: usize = 1;
/* WHY: Every column must get at least this many pixels regardless of available space,
 * to prevent short/narrow columns from collapsing to zero when a wide column dominates. */
const GUARANTEED_MIN_WIDTH: f32 = 40.0;

pub(crate) struct TableLayoutCalculator;

impl TableLayoutCalculator {
    /* WHY: Counts the maximum character width needed for each column by scanning
     * header + body spans. Matches pulldown.rs reference (line 1690-1704). */
    pub(crate) fn calculate_col_max_chars(table_data: &Table<'_>, num_cols: usize) -> Vec<usize> {
        let mut col_max_chars = vec![0usize; num_cols];
        for (col, max_chars) in col_max_chars.iter_mut().enumerate() {
            let mut current_max = 0;
            if let Some(header_col) = table_data.header.get(col) {
                let chars: usize = header_col.iter().map(|x| (x.1).1.len()).sum();
                current_max = current_max.max(chars);
            }
            for row in table_data.rows.iter() {
                if let Some(col_data) = row.get(col) {
                    let chars: usize = col_data.iter().map(|x| (x.1).1.len()).sum();
                    current_max = current_max.max(chars);
                }
            }
            *max_chars = current_max;
        }
        col_max_chars
    }

    /* WHY: Computes ideal_w = (chars * char_width_mul) + base_offset for each column,
     * sorted ascending. Matches pulldown.rs reference (line 1710-1716). */
    pub(crate) fn compute_ideal_widths(
        col_max_chars: &[usize],
        char_width_mul: f32,
        base_offset: f32,
    ) -> Vec<(f32, usize)> {
        let mut result: Vec<(f32, usize)> = col_max_chars
            .iter()
            .enumerate()
            .map(|(i, &len)| ((len as f32 * char_width_mul) + base_offset, i))
            .collect();
        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        result
    }

    /* WHY: Allocates column widths. Matches pulldown.rs reference EXACTLY (line 1718-1739).
     *
     * Strategy:
     * 1. If ALL columns fit within fair_w (= available_w / num_cols), give every column fair_w.
     *    This produces perfectly uniform column widths for simple tables (e.g. 5.1).
     * 2. Otherwise, process columns smallest-first:
     *    - If ideal_w < fair_share of remaining budget → use ideal_w
     *    - Otherwise → cap at fair_share
     *    This gives small columns their natural width and splits the rest evenly among big ones.
     *
     * Additionally, GUARANTEED_MIN_WIDTH prevents a column from collapsing to zero when a
     * single column's ideal_w dominates the available space (e.g. 5.5 Short|Long|Short). */
    pub(crate) fn compute_alloc_widths(
        num_cols: usize,
        available_w: f32,
        ideal_w_and_index: &[(f32, usize)],
    ) -> Vec<f32> {
        let mut col_alloc_width = vec![0.0; num_cols];

        /* WHY: Calculate the total ideal width of all columns. */
        let total_ideal_w: f32 = ideal_w_and_index.iter().map(|(w, _)| w).sum();

        /* WHY: Check if all columns easily fit the available space.
         * If they do, evenly distribute the extra space among all columns
         * so padding is uniform across columns (resolves 5.1 formatting). */
        if total_ideal_w <= available_w {
            let extra_space = (available_w - total_ideal_w) / (num_cols as f32);
            for &(ideal_w, col_idx) in ideal_w_and_index {
                col_alloc_width[col_idx] = ideal_w + extra_space;
            }
        } else {
            /* WHY: Not all fit → allocate greedily, smallest first (pulldown.rs reference). */
            let mut remaining_w = available_w;
            let mut remaining_cols = num_cols;

            for &(ideal_w, col_idx) in ideal_w_and_index {
                let fair_share = remaining_w / (remaining_cols as f32);

                let alloc_w = if ideal_w < fair_share {
                    ideal_w
                } else {
                    /* WHY: Cap at fair_share but guarantee at least GUARANTEED_MIN_WIDTH
                     * so narrow columns next to a dominating column stay visible (5.5 fix). */
                    let reserved_for_others =
                        GUARANTEED_MIN_WIDTH * (remaining_cols.saturating_sub(1)) as f32;
                    let max_for_current =
                        (remaining_w - reserved_for_others).max(GUARANTEED_MIN_WIDTH);
                    fair_share.min(max_for_current)
                };

                col_alloc_width[col_idx] = alloc_w;
                remaining_w -= alloc_w;
                remaining_cols -= MIN_FRACTION;
            }
        }

        col_alloc_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_alloc_widths_even_split() {
        let num_cols = 3;
        let available_w = 300.0;
        let ideal_w_and_index = vec![(100.0, 0), (100.0, 1), (100.0, 2)];
        let allocs =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);
        /* WHY: All fit under fair_w (100 each), so each gets fair_w = 100. */
        assert_eq!(allocs, vec![100.0, 100.0, 100.0]);
    }

    #[test]
    fn test_compute_alloc_widths_all_fit_uniform() {
        /* WHY: When all columns fit, they should ALL get fair_w regardless of individual ideal_w.
         * This is the 5.1 scenario: short data produces small ideal_w, but uniform distribution
         * is visually correct. */
        let num_cols = 3;
        let available_w = 600.0;
        let ideal_w_and_index = vec![(50.0, 0), (80.0, 1), (100.0, 2)];
        let allocs =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);
        assert_eq!(allocs, vec![173.33334, 203.33334, 223.33334]);
    }

    #[test]
    fn test_compute_alloc_widths_uneven_fit() {
        let num_cols = 3;
        let available_w = 300.0;
        /* WHY: Col 0 (ideal=50) fits under fair_share. Cols 1,2 (ideal=150) exceed it.
         * Col 0 gets 50, remaining 250 / 2 = 125 each. */
        let ideal_w_and_index = vec![(50.0, 0), (150.0, 1), (150.0, 2)];
        let allocs =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);
        assert_eq!(allocs, vec![50.0, 125.0, 125.0]);
    }

    #[test]
    fn test_compute_alloc_widths_all_constrained() {
        let num_cols = 2;
        let available_w = 100.0;
        let ideal_w_and_index = vec![(200.0, 0), (200.0, 1)];
        let allocs =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);
        /* WHY: Both exceed fair_w, so greedy path. Each gets 50.0 (fair_share). */
        assert_eq!(allocs, vec![50.0, 50.0]);
    }

    #[test]
    fn test_compute_alloc_widths_short_long_short() {
        /* WHY: Reproduces the 5.5 scenario: two very short columns + one extremely long column.
         * Short columns must remain visible (≥ GUARANTEED_MIN_WIDTH). */
        let num_cols = 3;
        let available_w = 500.0;
        let ideal_w_and_index = vec![(46.0, 0), (46.0, 2), (1576.0, 1)];
        let allocs =
            TableLayoutCalculator::compute_alloc_widths(num_cols, available_w, &ideal_w_and_index);
        assert!(
            allocs[0] >= 46.0,
            "Short col 0 should get at least its ideal width"
        );
        assert!(
            allocs[2] >= 46.0,
            "Short col 2 should get at least its ideal width"
        );
        assert!(
            allocs[1] >= GUARANTEED_MIN_WIDTH,
            "Long col 1 should get at least GUARANTEED_MIN_WIDTH"
        );
        let total: f32 = allocs.iter().sum();
        assert!(
            (total - available_w).abs() < 1.0,
            "Total allocation should approximately equal available_w"
        );
    }

    #[test]
    fn test_calculate_col_max_chars() {
        use pulldown_cmark::Event;
        let table_data = Table {
            header: vec![vec![(0, (Event::Text("Header".into()), 0..6))]],
            rows: vec![vec![vec![(1, (Event::Text("LongerText".into()), 0..10))]]],
        };
        let chars = TableLayoutCalculator::calculate_col_max_chars(&table_data, 1);
        assert_eq!(chars[0], 10);
    }

    #[test]
    fn test_compute_ideal_widths() {
        let chars = vec![5, 10, 3];
        let result = TableLayoutCalculator::compute_ideal_widths(&chars, 6.0, 16.0);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, 2);
        assert!((result[0].0 - 34.0).abs() < 0.1);
        assert_eq!(result[1].1, 0);
        assert!((result[1].0 - 46.0).abs() < 0.1);
        assert_eq!(result[2].1, 1);
        assert!((result[2].0 - 76.0).abs() < 0.1);
    }
}
