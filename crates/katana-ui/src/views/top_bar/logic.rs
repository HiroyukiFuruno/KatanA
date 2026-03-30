use crate::app_state::AppAction;

/// Compute drop-point positions from tab rects.
/// Returns Vec<(insert_idx, x_position)>.
pub fn compute_drop_points(tab_rects: &[(usize, eframe::egui::Rect)]) -> Vec<(usize, f32)> {
    let mut drop_points = Vec::new();
    if tab_rects.is_empty() {
        return drop_points;
    }
    for i in 0..tab_rects.len() {
        if i == 0 {
            drop_points.push((0, tab_rects[i].1.left()));
        } else {
            let prev_right = tab_rects[i - 1].1.right();
            let current_left = tab_rects[i].1.left();
            drop_points.push((i, (prev_right + current_left) / 2.0));
        }
    }
    drop_points.push((tab_rects.len(), tab_rects.last().unwrap().1.right()));
    drop_points
}

/// Find the best drop insertion index given a ghost center x position.
pub fn find_best_drop_index(drop_points: &[(usize, f32)], ghost_center_x: f32) -> Option<usize> {
    let mut best_dist = f32::MAX;
    let mut best_insert_idx = None;
    for (insert_idx, x) in drop_points {
        let dist = (ghost_center_x - x).abs();
        if dist < best_dist {
            best_dist = dist;
            best_insert_idx = Some(*insert_idx);
        }
    }
    best_insert_idx
}

/// Resolve a tab drag-drop into an AppAction (if needed).
pub fn resolve_drag_drop(
    src_idx: usize,
    ghost_center_x: f32,
    tab_rects: &[(usize, eframe::egui::Rect)],
) -> Option<AppAction> {
    let drop_points = compute_drop_points(tab_rects);
    if let Some(to) = find_best_drop_index(&drop_points, ghost_center_x) {
        if src_idx != to && src_idx + 1 != to {
            return Some(AppAction::ReorderDocument { from: src_idx, to });
        }
    }
    None
}

/// Build a tab display title from document metadata.
pub fn tab_display_title(
    original_filename: &str,
    is_changelog: bool,
    is_dirty: bool,
    is_pinned: bool,
) -> String {
    let filename = if is_changelog {
        original_filename.to_string()
    } else if original_filename.starts_with("CHANGELOG_v") && original_filename.ends_with(".md") {
        let ver = original_filename
            .trim_start_matches("CHANGELOG_v")
            .trim_end_matches(".md");
        format!("📄 {} {}", crate::i18n::get().menu.release_notes, ver)
    } else {
        original_filename.to_string()
    };
    let dirty_suffix = if is_dirty { " *" } else { "" };
    if is_pinned {
        format!("📌 {filename}{dirty_suffix}")
    } else {
        format!("{filename}{dirty_suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;

    fn make_rects(positions: &[(f32, f32)]) -> Vec<(usize, egui::Rect)> {
        positions
            .iter()
            .enumerate()
            .map(|(i, (left, right))| {
                (
                    i,
                    egui::Rect::from_min_max(egui::pos2(*left, 0.0), egui::pos2(*right, 20.0)),
                )
            })
            .collect()
    }

    #[test]
    fn compute_drop_points_empty() {
        assert!(compute_drop_points(&[]).is_empty());
    }

    #[test]
    fn compute_drop_points_single_tab() {
        let rects = make_rects(&[(10.0, 100.0)]);
        let points = compute_drop_points(&rects);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], (0, 10.0));
        assert_eq!(points[1], (1, 100.0));
    }

    #[test]
    fn compute_drop_points_two_tabs() {
        let rects = make_rects(&[(10.0, 100.0), (110.0, 200.0)]);
        let points = compute_drop_points(&rects);
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], (0, 10.0));
        assert_eq!(points[1], (1, 105.0));
        assert_eq!(points[2], (2, 200.0));
    }

    #[test]
    fn find_best_drop_index_nearest() {
        let points = vec![(0, 10.0), (1, 105.0), (2, 200.0)];
        assert_eq!(find_best_drop_index(&points, 50.0), Some(0));
        assert_eq!(find_best_drop_index(&points, 110.0), Some(1));
        assert_eq!(find_best_drop_index(&points, 190.0), Some(2));
    }

    #[test]
    fn find_best_drop_index_empty() {
        assert_eq!(find_best_drop_index(&[], 50.0), None);
    }

    #[test]
    fn resolve_drag_drop_same_position_no_action() {
        let rects = make_rects(&[(10.0, 100.0), (110.0, 200.0)]);
        assert!(resolve_drag_drop(0, 30.0, &rects).is_none());
    }

    #[test]
    fn resolve_drag_drop_adjacent_no_action() {
        let rects = make_rects(&[(10.0, 100.0), (110.0, 200.0)]);
        assert!(resolve_drag_drop(0, 105.0, &rects).is_none());
    }

    #[test]
    fn resolve_drag_drop_reorder_action() {
        let rects = make_rects(&[(10.0, 100.0), (110.0, 200.0), (210.0, 300.0)]);
        let action = resolve_drag_drop(0, 260.0, &rects);
        assert!(matches!(
            action,
            Some(AppAction::ReorderDocument { from: 0, to: 3 })
        ));
    }

    #[test]
    fn tab_display_title_normal() {
        let title = tab_display_title("readme.md", false, false, false);
        assert_eq!(title, "readme.md");
    }

    #[test]
    fn tab_display_title_dirty() {
        let title = tab_display_title("readme.md", false, true, false);
        assert_eq!(title, "readme.md *");
    }

    #[test]
    fn tab_display_title_pinned() {
        let title = tab_display_title("readme.md", false, false, true);
        assert_eq!(title, "📌 readme.md");
    }

    #[test]
    fn tab_display_title_pinned_dirty() {
        let title = tab_display_title("readme.md", false, true, true);
        assert_eq!(title, "📌 readme.md *");
    }

    #[test]
    fn tab_display_title_changelog() {
        let title = tab_display_title("ChangeLog", true, false, false);
        assert_eq!(title, "ChangeLog");
    }
}
