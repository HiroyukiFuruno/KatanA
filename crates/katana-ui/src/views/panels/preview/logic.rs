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
        preview: &crate::preview_pane::PreviewPane,
        row_height: f32,
    ) -> Option<f32> {
        let consuming_editor =
            scroll_sync && scroll.source == crate::app_state::ScrollSource::Editor;
        if consuming_editor {
            return Some(scroll.mapper.logical_to_preview(scroll.logical_position));
        }

        let target_line = scroll.scroll_to_line?;
        if scroll.preview_search_scroll_pending {
            return None;
        }

        if scroll_sync {
            let editor_y = target_line as f32 * row_height;
            scroll.logical_position = scroll.mapper.editor_to_logical(editor_y);
            return Some(scroll.mapper.logical_to_preview(scroll.logical_position));
        }

        for (span, rect) in &preview.heading_anchors {
            if span.contains(&target_line) || span.start >= target_line {
                return Some((rect.min.y - preview.content_top_y).max(0.0));
            }
        }

        for (span, rect) in &preview.block_anchors {
            if span.contains(&target_line) || span.start >= target_line {
                return Some((rect.min.y - preview.content_top_y).max(0.0));
            }
        }

        if let Some((_, rect)) = preview.heading_anchors.last() {
            Some((rect.min.y - preview.content_top_y).max(0.0))
        } else {
            Some(0.0)
        }
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

        let mut computed_anchors =
            Vec::with_capacity(preview.heading_anchors.len() + preview.block_anchors.len());
        for (span, rect) in &preview.heading_anchors {
            let p_y = (rect.min.y - preview.content_top_y).max(0.0);
            computed_anchors.push((span.clone(), p_y));
        }
        for (span, rect) in &preview.block_anchors {
            let p_y = (rect.min.y - preview.content_top_y).max(0.0);
            computed_anchors.push((span.clone(), p_y));
        }

        scroll.mapper = crate::state::scroll_sync::ScrollMapper::build(
            scroll.editor_max,
            scroll.preview_max,
            row_height,
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
    }
}
