use super::types::*;
use crate::shell_ui::TOC_HEADING_VISIBILITY_THRESHOLD;
use eframe::egui;

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
fn split_mode_preview_and_editor_agree_on_click_target() {
    let heading_positions = &[
        (0, 0.0, 0usize, 1usize),
        (10, 150.0, 10, 11),
        (30, 400.0, 30, 31),
        (60, 700.0, 60, 61),
        (100, 1200.0, 100, 101),
    ];

    let clicked_index = 3;
    let clicked_y = heading_positions[clicked_index].1;
    let clicked_line = heading_positions[clicked_index].0;

    let outline_items = make_outline_items(heading_positions.len());

    let anchors: Vec<_> = heading_positions
        .iter()
        .enumerate()
        .map(|(idx, (line, y, _, _))| {
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, *y), egui::vec2(100.0, 20.0));
            crate::preview_pane::types::DocumentAnchorMapItem {
                kind: katana_core::markdown::outline::AnchorKind::Heading,
                index: Some(idx),
                line_span: *line..*line + 1,
                rect: Some(rect),
            }
        })
        .collect();
    let preview_threshold = clicked_y + TOC_HEADING_VISIBILITY_THRESHOLD;
    let preview_active =
        TocPanel::find_active_toc_index_preview(&outline_items, &anchors, preview_threshold);

    let anchor_map: Vec<_> = heading_positions
        .iter()
        .enumerate()
        .map(
            |(idx, (_, _, start, end))| crate::preview_pane::types::DocumentAnchorMapItem {
                kind: katana_core::markdown::outline::AnchorKind::Heading,
                index: Some(idx),
                line_span: *start..*end,
                rect: None,
            },
        )
        .collect();
    const ROW_HEIGHT: f32 = 16.0;
    let current_line = clicked_line as f32;
    let threshold_lines = TOC_HEADING_VISIBILITY_THRESHOLD / ROW_HEIGHT;
    let editor_active = TocPanel::find_active_toc_index_editor(
        &outline_items,
        &anchor_map,
        current_line + threshold_lines,
    );

    assert_eq!(preview_active, clicked_index);
    assert_eq!(editor_active, clicked_index);
}
