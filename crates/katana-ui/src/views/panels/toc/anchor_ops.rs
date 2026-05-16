use super::types::*;
use eframe::egui;

impl<'a> TocPanel<'a> {
    pub(crate) fn resolve_toc_anchor_candidate(
        view_mode: crate::app_state::ViewMode,
        scroll: &crate::app_state::ScrollState,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        visible_rect: Option<egui::Rect>,
        row_height: f32,
    ) -> Option<TocAnchorCandidate> {
        match view_mode {
            crate::app_state::ViewMode::CodeOnly => {
                Self::resolve_editor_viewport_anchor(scroll, anchor_map, row_height)
            }
            crate::app_state::ViewMode::PreviewOnly => {
                Self::resolve_preview_hover_anchor(scroll, anchor_map)
                    .or_else(|| Self::resolve_preview_viewport_anchor(visible_rect, anchor_map))
            }
            crate::app_state::ViewMode::Split => {
                Self::resolve_preview_hover_anchor(scroll, anchor_map).or_else(|| {
                    Self::resolve_split_viewport_anchor(
                        scroll,
                        anchor_map,
                        visible_rect,
                        row_height,
                    )
                })
            }
        }
    }

    pub(crate) fn record_toc_click_anchor(
        toc: &mut crate::state::toc::TocState,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        toc_index: usize,
    ) {
        if let Some(item) = anchor_map.iter().find(|item| item.index == Some(toc_index))
            && let Some(toc_index) = item.toc_index
        {
            toc.update_current(
                item.anchor_index,
                toc_index,
                crate::state::toc::TocCurrentOrigin::TocClick,
            );
        }
    }

    pub(crate) fn active_toc_index_from_anchor_state(
        &mut self,
        row_height: f32,
        now: f64,
    ) -> usize {
        let candidate = Self::resolve_toc_anchor_candidate(
            self.state.active_view_mode(),
            &self.state.scroll,
            &self.preview.anchor_map,
            self.preview.visible_rect,
            row_height,
        );
        if let Some(candidate) = candidate {
            self.state.toc.apply_viewport_candidate(candidate, now);
        }
        self.state
            .toc
            .current
            .map(|anchor| anchor.toc_index)
            .or_else(|| self.preview.outline_items.first().map(|item| item.index))
            .unwrap_or(0)
    }

    fn resolve_editor_viewport_anchor(
        scroll: &crate::app_state::ScrollState,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        row_height: f32,
    ) -> Option<TocAnchorCandidate> {
        let line = Self::editor_logical_threshold(scroll, row_height)
            .floor()
            .max(0.0) as usize;
        Self::candidate_for_line(
            anchor_map,
            line,
            crate::state::toc::TocCurrentOrigin::EditorViewport,
        )
    }

    fn resolve_preview_hover_anchor(
        scroll: &crate::app_state::ScrollState,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
    ) -> Option<TocAnchorCandidate> {
        let hovered = scroll.hovered_preview_lines.first()?;
        Self::candidate_for_line_range(
            anchor_map,
            hovered,
            crate::state::toc::TocCurrentOrigin::PreviewHover,
        )
    }

    fn resolve_split_viewport_anchor(
        scroll: &crate::app_state::ScrollState,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        visible_rect: Option<egui::Rect>,
        row_height: f32,
    ) -> Option<TocAnchorCandidate> {
        match scroll.source {
            crate::app_state::ScrollSource::Editor => {
                Self::resolve_editor_viewport_anchor(scroll, anchor_map, row_height)
            }
            crate::app_state::ScrollSource::Preview => {
                Self::resolve_preview_viewport_anchor(visible_rect, anchor_map)
            }
            crate::app_state::ScrollSource::Neither => None,
        }
    }

    fn resolve_preview_viewport_anchor(
        visible_rect: Option<egui::Rect>,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
    ) -> Option<TocAnchorCandidate> {
        let visible_rect = visible_rect?;
        let threshold = visible_rect.min.y + crate::shell_ui::TOC_HEADING_VISIBILITY_THRESHOLD;
        let item = Self::find_anchor_for_preview_threshold(anchor_map, threshold)?;
        Self::candidate_from_item(item, crate::state::toc::TocCurrentOrigin::PreviewViewport)
    }

    fn candidate_for_line(
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        line: usize,
        source: crate::state::toc::TocCurrentOrigin,
    ) -> Option<TocAnchorCandidate> {
        let item = Self::find_anchor_for_line(anchor_map, line)?;
        Self::candidate_from_item(item, source)
    }

    fn candidate_for_line_range(
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        line_range: &std::ops::Range<usize>,
        source: crate::state::toc::TocCurrentOrigin,
    ) -> Option<TocAnchorCandidate> {
        let item = Self::find_anchor_for_line_range(anchor_map, line_range)?;
        Self::candidate_from_item(item, source)
    }

    fn candidate_from_item(
        item: &crate::preview_pane::types::DocumentAnchorMapItem,
        source: crate::state::toc::TocCurrentOrigin,
    ) -> Option<TocAnchorCandidate> {
        Some(TocAnchorCandidate {
            anchor_index: item.anchor_index,
            toc_index: item.toc_index?,
            source,
        })
    }
}
