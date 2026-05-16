use super::types::*;
use crate::shell_ui::TOC_HEADING_VISIBILITY_THRESHOLD;
use eframe::egui;

mod anchor_state_tests;
mod render_tests;

/// Helper: build anchor_map items for preview mode testing (includes rects)
fn make_anchor_map_with_rects(
    entries: &[(usize, f32)],
) -> Vec<crate::preview_pane::types::DocumentAnchorMapItem> {
    const MOCK_ROW_WIDTH: f32 = 100.0;
    const MOCK_ROW_HEIGHT: f32 = 20.0;
    entries
        .iter()
        .enumerate()
        .map(|(idx, (line, y))| {
            let rect = egui::Rect::from_min_size(
                egui::pos2(0.0, *y),
                egui::vec2(MOCK_ROW_WIDTH, MOCK_ROW_HEIGHT),
            );
            crate::preview_pane::types::DocumentAnchorMapItem {
                anchor_index: idx,
                toc_index: Some(idx),
                kind: katana_core::markdown::outline::AnchorKind::Heading,
                index: Some(idx),
                line_span: *line..*line + 1,
                rect: Some(rect),
            }
        })
        .collect()
}

/// Helper: build anchor_map items for editor mode testing.
fn make_anchor_map(
    entries: &[(usize, usize)],
) -> Vec<crate::preview_pane::types::DocumentAnchorMapItem> {
    entries
        .iter()
        .enumerate()
        .map(
            |(idx, (line_start, line_end))| crate::preview_pane::types::DocumentAnchorMapItem {
                anchor_index: idx,
                toc_index: Some(idx),
                kind: katana_core::markdown::outline::AnchorKind::Heading,
                index: Some(idx),
                line_span: *line_start..*line_end,
                rect: None,
            },
        )
        .collect()
}

fn make_outline_items(count: usize) -> Vec<katana_core::markdown::outline::OutlineItem> {
    const LINE_MULTIPLIER: usize = 10;
    (0..count)
        .map(|i| katana_core::markdown::outline::OutlineItem {
            level: 1,
            text: format!("Heading {}", i),
            index: i,
            line_start: i * LINE_MULTIPLIER,
            line_end: i * LINE_MULTIPLIER + 1,
        })
        .collect()
}

#[test]
fn preview_click_heading_highlights_correct_index() {
    let anchors = make_anchor_map_with_rects(&[(0, 0.0), (10, 200.0), (30, 500.0)]);
    let outline_items = make_outline_items(3);
    let threshold = 500.0 + TOC_HEADING_VISIBILITY_THRESHOLD;
    let active = TocPanel::find_active_toc_index_preview(&outline_items, &anchors, threshold);
    assert_eq!(active, 2);
}

#[test]
fn preview_click_middle_heading_highlights_correctly() {
    let anchors = make_anchor_map_with_rects(&[(0, 0.0), (10, 200.0), (30, 500.0)]);
    let outline_items = make_outline_items(3);
    let threshold = 200.0 + TOC_HEADING_VISIBILITY_THRESHOLD;
    let active = TocPanel::find_active_toc_index_preview(&outline_items, &anchors, threshold);
    assert_eq!(active, 1);
}

#[test]
fn preview_at_top_highlights_first_heading() {
    let anchors = make_anchor_map_with_rects(&[(0, 0.0), (10, 200.0), (30, 500.0)]);
    let outline_items = make_outline_items(3);
    let threshold = 0.0 + TOC_HEADING_VISIBILITY_THRESHOLD;
    let active = TocPanel::find_active_toc_index_preview(&outline_items, &anchors, threshold);
    assert_eq!(active, 0);
}

#[test]
fn editor_click_heading_highlights_correct_index() {
    let anchor_map = make_anchor_map(&[(0, 1), (52, 53)]);
    let outline_items = make_outline_items(2);
    const ROW_HEIGHT: f32 = 16.0;
    let current_line = 52.0;
    let threshold_lines = TOC_HEADING_VISIBILITY_THRESHOLD / ROW_HEIGHT;
    let active = TocPanel::find_active_toc_index_editor(
        &outline_items,
        &anchor_map,
        current_line + threshold_lines,
    );
    assert_eq!(active, 1);
}

#[test]
fn editor_at_top_highlights_first_heading() {
    let anchor_map = make_anchor_map(&[(0, 1), (52, 53)]);
    let outline_items = make_outline_items(2);
    const ROW_HEIGHT: f32 = 16.0;
    let current_line = 0.0;
    let threshold_lines = TOC_HEADING_VISIBILITY_THRESHOLD / ROW_HEIGHT;
    let active = TocPanel::find_active_toc_index_editor(
        &outline_items,
        &anchor_map,
        current_line + threshold_lines,
    );
    assert_eq!(active, 0);
}

#[test]
fn editor_threshold_prefers_pending_scroll_target() {
    let scroll = crate::app_state::ScrollState {
        editor_y: 0.0,
        scroll_to_line: Some(52),
        ..Default::default()
    };

    let threshold = TocPanel::editor_logical_threshold(&scroll, 16.0);

    assert_eq!(threshold, 53.0);
}

#[test]
fn editor_threshold_prefers_pending_toc_scroll_target() {
    let scroll = crate::app_state::ScrollState {
        editor_y: 0.0,
        toc_scroll_to_line: Some(52),
        ..Default::default()
    };

    let threshold = TocPanel::editor_logical_threshold(&scroll, 16.0);

    assert_eq!(threshold, 53.0);
}

#[test]
fn editor_threshold_uses_line_anchors_when_soft_wrap_pushes_heading_down() {
    let scroll = crate::app_state::ScrollState {
        editor_y: 60.0,
        editor_line_anchors: vec![0.0, 16.0, 32.0, 80.0, 96.0],
        ..Default::default()
    };
    let outline_items = make_outline_items(2);
    let anchor_map = make_anchor_map(&[(0, 1), (4, 5)]);

    let threshold = TocPanel::editor_logical_threshold(&scroll, 16.0);
    let active = TocPanel::find_active_toc_index_editor(&outline_items, &anchor_map, threshold);

    assert_eq!(active, 0);
}

#[test]
fn unchanged_active_item_does_not_request_toc_auto_scroll() {
    assert!(!TocPanel::should_auto_scroll_active_item(Some(1), 1));
}

#[test]
fn changed_active_item_requests_toc_auto_scroll() {
    assert!(TocPanel::should_auto_scroll_active_item(Some(1), 15));
    assert!(TocPanel::should_auto_scroll_active_item(None, 1));
}
