use crate::state::toc::TocCurrentOrigin;
use crate::views::panels::toc::TocPanel;
use eframe::egui;

use super::helpers::{
    active_app_state_with_active_document, active_index_for_code_with_click,
    active_index_for_preview_with_click,
};

#[test]
fn toc_click_sets_shared_current_for_code_and_preview() {
    let anchor_map =
        super::super::make_anchor_map_with_rects(&[(0, 0.0), (40, 120.0), (80, 240.0)]);

    let (code_index, code_state) = active_index_for_code_with_click(1, 60.0, &anchor_map, 3);
    assert_eq!(code_index, 1);
    assert_eq!(code_state.toc.current.map(|c| c.toc_index), Some(1));

    let (preview_index, preview_state) = active_index_for_preview_with_click(
        1,
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 500.0)),
        &anchor_map,
        3,
    );
    assert_eq!(preview_index, 1);
    assert_eq!(preview_state.toc.current.map(|c| c.toc_index), Some(1));
    assert_eq!(
        code_state.toc.current.map(|c| c.toc_index),
        preview_state.toc.current.map(|c| c.toc_index)
    );
}

#[test]
fn preview_first_viewport_observation_keeps_toc_click_current() {
    let anchor_map = super::super::make_anchor_map_with_rects(&[(0, 0.0), (40, 120.0)]);
    let (active, state) = active_index_for_preview_with_click(
        1,
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 80.0)),
        &anchor_map,
        2,
    );

    assert_eq!(active, 1);
    assert_eq!(
        state.toc.current.map(|c| c.origin),
        Some(TocCurrentOrigin::TocClick)
    );
}

#[test]
fn preview_hover_after_toc_click_updates_current() {
    let anchor_map =
        super::super::make_anchor_map_with_rects(&[(0, 0.0), (40, 120.0), (80, 240.0)]);
    let mut state = active_app_state_with_active_document();
    let mut preview = crate::preview_pane::PreviewPane::default();
    preview.anchor_map = anchor_map;
    preview.outline_items = super::super::make_outline_items(3);
    state.set_active_view_mode(crate::app_state::ViewMode::PreviewOnly);
    assert_eq!(
        state.active_view_mode(),
        crate::app_state::ViewMode::PreviewOnly
    );

    state.scroll.hovered_preview_lines = vec![0..1];
    TocPanel::record_toc_click_anchor(&mut state.toc, &preview.anchor_map, 1);

    let active_early = {
        let mut panel = TocPanel::new(&mut preview, &mut state);
        panel.active_toc_index_from_anchor_state(20.0, 0.0)
    };
    let active_late = {
        let mut panel = TocPanel::new(&mut preview, &mut state);
        panel.active_toc_index_from_anchor_state(20.0, 0.026)
    };

    assert_eq!(active_early, 1);
    assert_eq!(active_late, 0);
    assert_eq!(
        state.toc.current.map(|c| c.origin),
        Some(TocCurrentOrigin::PreviewHover)
    );
    assert_eq!(state.toc.current.map(|c| c.toc_index), Some(0));
}

#[test]
fn code_first_viewport_observation_keeps_toc_click_current() {
    let anchor_map = super::super::make_anchor_map(&[(0, 10), (20, 30)]);
    let (active, state) = active_index_for_code_with_click(1, 10.0, &anchor_map, 2);

    assert_eq!(active, 1);
    assert_eq!(
        state.toc.current.map(|c| c.origin),
        Some(TocCurrentOrigin::TocClick)
    );
}
