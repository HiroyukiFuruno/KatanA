use crate::app_state::AppState;
use crate::app_state::ViewMode;
use crate::preview_pane::PreviewPane;
use eframe::egui;

pub fn active_app_state_with_active_document() -> AppState {
    let mut state = AppState::new(
        Default::default(),
        Default::default(),
        Default::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            "toc-state-test.md",
            "# toc-state\n",
        ));
    state.document.active_doc_idx = Some(0);
    state
}

pub fn make_preview_candidate(
    visible_rect: egui::Rect,
    anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
) -> crate::state::toc::TocAnchorCandidate {
    let scroll = crate::app_state::ScrollState::default();
    let candidate_rect = Some(visible_rect);
    crate::views::panels::toc::TocPanel::resolve_toc_anchor_candidate(
        ViewMode::PreviewOnly,
        &scroll,
        anchor_map,
        candidate_rect,
        20.0,
    )
    .expect("candidate should be resolved")
}

pub fn make_code_view_candidate(
    scroll_y: f32,
    editor_line_anchors: &[f32],
    anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
) -> crate::state::toc::TocAnchorCandidate {
    let scroll = crate::app_state::ScrollState {
        editor_line_anchors: editor_line_anchors.to_vec(),
        editor_y: scroll_y,
        ..Default::default()
    };
    let preview = PreviewPane::default();
    crate::views::panels::toc::TocPanel::resolve_toc_anchor_candidate(
        ViewMode::CodeOnly,
        &scroll,
        anchor_map,
        preview.visible_rect,
        20.0,
    )
    .expect("candidate should be resolved")
}

pub fn active_index_for_code_with_click(
    click_toc_index: usize,
    scroll_y: f32,
    anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
    outline_count: usize,
) -> (usize, AppState) {
    let mut state = active_app_state_with_active_document();
    let mut preview = PreviewPane::default();
    preview.anchor_map = anchor_map.to_vec();
    preview.outline_items = crate::views::panels::toc::tests::make_outline_items(outline_count);
    state.set_active_view_mode(ViewMode::CodeOnly);
    assert_eq!(state.active_view_mode(), ViewMode::CodeOnly);
    state.scroll.editor_line_anchors = vec![0.0, 20.0, 40.0];
    state.scroll.editor_y = scroll_y;

    crate::views::panels::toc::TocPanel::record_toc_click_anchor(
        &mut state.toc,
        &preview.anchor_map,
        click_toc_index,
    );
    let active_index = {
        let mut panel = crate::views::panels::toc::TocPanel::new(&mut preview, &mut state);
        panel.active_toc_index_from_anchor_state(20.0, 0.0)
    };
    (active_index, state)
}

pub fn active_index_for_preview_with_click(
    click_toc_index: usize,
    visible_rect: egui::Rect,
    anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
    outline_count: usize,
) -> (usize, AppState) {
    let mut state = active_app_state_with_active_document();
    let mut preview = PreviewPane::default();
    preview.anchor_map = anchor_map.to_vec();
    preview.visible_rect = Some(visible_rect);
    preview.outline_items = crate::views::panels::toc::tests::make_outline_items(outline_count);

    state.set_active_view_mode(ViewMode::PreviewOnly);
    assert_eq!(state.active_view_mode(), ViewMode::PreviewOnly);
    crate::views::panels::toc::TocPanel::record_toc_click_anchor(
        &mut state.toc,
        &preview.anchor_map,
        click_toc_index,
    );

    let active_index = {
        let mut panel = crate::views::panels::toc::TocPanel::new(&mut preview, &mut state);
        panel.active_toc_index_from_anchor_state(20.0, 0.0)
    };
    (active_index, state)
}
