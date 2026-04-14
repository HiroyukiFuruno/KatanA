use super::types::*;
use crate::app_state::AppAction;
use eframe::egui;

impl PreviewLogicOps {
    pub fn preview_panel_id(path: Option<&std::path::Path>, base: &'static str) -> egui::Id {
        match path {
            Some(path) => egui::Id::new((base, path)),
            None => egui::Id::new(base),
        }
    }

    pub fn invalidate_preview_image_cache(ctx: &egui::Context, action: &AppAction) {
        if matches!(action, AppAction::RefreshDiagrams) {
            ctx.forget_all_images();
        }
    }

    pub fn compute_forced_offset(
        scroll_sync: bool,
        scroll: &mut crate::app_state::ScrollState,
        _preview: &crate::preview_pane::PreviewPane,
        _row_height: f32,
        _inner_height: f32,
    ) -> Option<f32> {
        if !scroll_sync {
            return None;
        }
        let consuming_editor = scroll_sync
            && scroll.source == crate::app_state::ScrollSource::Editor
            && scroll.scroll_to_line.is_none();
        if consuming_editor {
            let raw_offset = scroll.mapper.logical_to_preview(scroll.logical_position);
            return Some(scroll.mapper.snap_to_heading_preview(raw_offset));
        }

        None
    }

    /// Compute the precise preview ScrollArea vertical offset to jump to a heading.
    ///
    /// Uses the previous frame's `heading_anchors` (screen-space rects) and
    /// `content_top_y` to derive the correct scroll offset. Returns `None` when
    /// anchors are not yet available (first render or document just changed).
    pub fn heading_scroll_offset(
        heading_index: usize,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        content_top_y: f32,
    ) -> Option<f32> {
        let item = anchor_map.iter().find(|a| a.index == Some(heading_index))?;
        let rect = item.rect?;
        /* WHY: rect.min.y is screen-space. Subtracting content_top_y converts it   */
        /* WHY: to the ScrollArea's virtual-space offset (scroll_offset = 0 when at  */
        /* WHY: the very top of the content, before any Frame/padding offsets).      */
        /* WHY: We clamp to 0.0 to avoid negative offsets on the first heading.     */
        Some((rect.min.y - content_top_y).max(0.0))
    }

    pub fn update_scroll_sync(
        scroll: &mut crate::app_state::ScrollState,
        preview: &crate::preview_pane::PreviewPane,
        row_height: f32,
        content_height: f32,
        inner_height: f32,
        offset_y: f32,
    ) {
        let max_scroll = (content_height - inner_height).max(0.0);
        scroll.preview_max = max_scroll;
        scroll.preview_y = offset_y;

        let mut computed_anchors = Vec::with_capacity(preview.anchor_map.len());
        for item in &preview.anchor_map {
            if let Some(rect) = item.rect {
                // WHY: Use physical editor line Y positions for ultra-high precision.
                // This eliminates drift caused by soft-wrapping in the editor.
                // If physical data is not yet available, we fallback to row-height estimation.
                let editor_y = scroll
                    .editor_line_anchors
                    .get(item.line_span.start)
                    .cloned()
                    .unwrap_or_else(|| item.line_span.start as f32 * row_height);

                let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                computed_anchors.push((editor_y, p_y));
            }
        }

        scroll.mapper = crate::state::scroll_sync::ScrollMapper::build(
            scroll.editor_max,
            scroll.preview_max,
            &computed_anchors,
        );

        let consuming_editor = scroll.source == crate::app_state::ScrollSource::Editor;
        if consuming_editor {
            scroll.source = crate::app_state::ScrollSource::Neither;
            return;
        }

        if max_scroll <= 0.0 {
            return;
        }

        if scroll.preview_echo.is_echo(offset_y) {
            return;
        }

        let next_logical = scroll.mapper.preview_to_logical(offset_y);
        if next_logical != scroll.logical_position {
            scroll.logical_position = next_logical;
            scroll.source = crate::app_state::ScrollSource::Preview;
        }

        /* WHY: Store the calculated ghost space for the UI to apply in the next frame. */
        /* We need a Ui context to access temp storage. Since we don't have it here, */
        /* we'll assume the caller (usually a view that has access to AppState/Ui) */
        /* will fetch it from scroll.mapper. We'll add it to ScrollState for easier access if needed, */
        /* but for now we'll just let the UI call the mapper's method directly if it has access. */
    }
}
