use crate::app_state::AppState;
use crate::app_state::ViewMode;
use crate::preview_pane::PreviewPane;
use crate::state::toc::TocCurrentOrigin;
use crate::views::panels::toc::TocPanel;

use super::helpers::{
    active_app_state_with_active_document, make_code_view_candidate, make_preview_candidate,
};

#[test]
fn preview_viewport_change_after_first_updates_current() {
    let anchor_map =
        super::super::make_anchor_map_with_rects(&[(0, 0.0), (40, 120.0), (80, 240.0)]);
    let mut state = active_app_state_with_active_document();
    let mut preview = PreviewPane::default();
    preview.anchor_map = anchor_map.clone();
    preview.outline_items = super::super::make_outline_items(3);
    state.set_active_view_mode(ViewMode::PreviewOnly);
    assert_eq!(state.active_view_mode(), ViewMode::PreviewOnly);
    preview.visible_rect = Some(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(100.0, 80.0),
    ));
    TocPanel::record_toc_click_anchor(&mut state.toc, &preview.anchor_map, 1);

    {
        let mut panel = TocPanel::new(&mut preview, &mut state);
        panel.active_toc_index_from_anchor_state(20.0, 0.0);
    }

    let second = make_preview_candidate(
        egui::Rect::from_min_size(egui::pos2(0.0, 200.0), egui::vec2(100.0, 80.0)),
        &anchor_map,
    );
    assert!(!state.toc.apply_viewport_candidate(second, 0.0));
    assert_eq!(state.toc.current.map(|c| c.toc_index), Some(1));
    assert!(state.toc.apply_viewport_candidate(second, 0.026));
    assert_eq!(state.toc.current.map(|c| c.toc_index), Some(2));
    assert_eq!(
        state.toc.current.map(|c| c.origin),
        Some(TocCurrentOrigin::PreviewViewport)
    );
}

#[test]
fn code_viewport_change_after_first_updates_current() {
    let anchor_map = super::super::make_anchor_map(&[(0, 10), (20, 30), (40, 60)]);
    let mut state = active_app_state_with_active_document();
    let mut preview = PreviewPane::default();
    preview.anchor_map = anchor_map.clone();
    preview.outline_items = super::super::make_outline_items(3);
    state.set_active_view_mode(ViewMode::CodeOnly);
    assert_eq!(state.active_view_mode(), ViewMode::CodeOnly);
    state.scroll.editor_line_anchors = (0..=60).map(|i| i as f32).collect();
    state.scroll.editor_y = 1.0;
    TocPanel::record_toc_click_anchor(&mut state.toc, &preview.anchor_map, 1);

    let first = make_code_view_candidate(1.0, &state.scroll.editor_line_anchors, &anchor_map);
    let second = make_code_view_candidate(50.0, &state.scroll.editor_line_anchors, &anchor_map);
    assert!(!state.toc.apply_viewport_candidate(first, 0.0));
    assert_ne!(first.anchor_index, second.anchor_index);
    assert!(!state.toc.apply_viewport_candidate(second, 0.0));
    assert!(state.toc.apply_viewport_candidate(second, 0.026));
    state.scroll.editor_y = 50.0;

    {
        let mut panel = TocPanel::new(&mut preview, &mut state);
        let active_index = panel.active_toc_index_from_anchor_state(20.0, 0.05);
        assert_eq!(active_index, 2);
    }

    assert_eq!(
        state.toc.current.map(|c| c.origin),
        Some(TocCurrentOrigin::EditorViewport)
    );
}

#[test]
fn scroll_state_reset_for_document_change_does_not_touch_toc_current() {
    let mut state = AppState::new(
        Default::default(),
        Default::default(),
        Default::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let mut preview = PreviewPane::default();
    preview.anchor_map = super::super::make_anchor_map(&[(0, 10), (20, 30)]);
    TocPanel::record_toc_click_anchor(&mut state.toc, &preview.anchor_map, 1);

    let current = state.toc.current;
    state.scroll.scroll_to_line = Some(10);
    state.scroll.toc_scroll_to_line = Some(20);
    state.scroll.last_scroll_to_line = Some(5);
    state.scroll.editor_y = 40.0;
    state.scroll.preview_y = 12.0;
    state.scroll.reset_for_document_change();

    assert_eq!(state.toc.current, current);
}

#[test]
fn toc_state_reset_for_document_change_clears_current() {
    let mut state = AppState::new(
        Default::default(),
        Default::default(),
        Default::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let mut preview = PreviewPane::default();
    preview.anchor_map = super::super::make_anchor_map(&[(0, 10)]);
    TocPanel::record_toc_click_anchor(&mut state.toc, &preview.anchor_map, 0);

    state.toc.reset_for_document_change();
    assert_eq!(state.toc.current, None);
}
