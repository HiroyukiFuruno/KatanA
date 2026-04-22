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

    /* Compute the precise preview ScrollArea vertical offset to jump to a heading.
    Uses the previous frame's `heading_anchors` (screen-space rects) and
    `content_top_y` to derive the correct scroll offset. Returns `None` when
    anchors are not yet available (first render or document just changed). */
    pub fn heading_scroll_offset(
        heading_index: usize,
        anchor_map: &[crate::preview_pane::types::DocumentAnchorMapItem],
        content_top_y: f32,
    ) -> Option<f32> {
        let item = anchor_map.iter().find(|a| a.index == Some(heading_index))?;
        let rect = item.rect?;
        /* WHY: rect.min.y is screen-space. Subtracting content_top_y converts it   */
        /* to the ScrollArea's virtual-space offset (scroll_offset = 0 when at      */
        /* the very top of the content, before any Frame/padding offsets).         */
        /* We clamp to 0.0 to avoid negative offsets on the first heading.          */
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
                /* WHY: Use physical editor line Y positions for ultra-high precision. */
                /* This eliminates drift caused by soft-wrapping in the editor. Fallback to row-height. */
                let editor_y = scroll
                    .editor_line_anchors
                    .get(item.line_span.start)
                    .cloned()
                    .unwrap_or(item.line_span.start as f32 * row_height);

                let p_y = (rect.min.y - preview.content_top_y).max(0.0);
                computed_anchors.push((editor_y, p_y));
            }
        }

        scroll.mapper = crate::state::scroll_sync::ScrollMapper::build(
            scroll.editor_max,
            scroll.preview_max,
            &computed_anchors,
        );

        let consuming = scroll.source == crate::app_state::ScrollSource::Editor;
        if consuming {
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

    pub fn render_preview_top_padding(ui: &mut eframe::egui::Ui) {
        const PREVIEW_PANE_TOP_BOTTOM_PADDING: f32 = 4.0;
        ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
    }

    pub fn render_preview_bottom_padding(
        ui: &mut eframe::egui::Ui,
        scroll: &crate::app_state::ScrollState,
    ) {
        /* WHY: Apply Ghost Space and viewport padding for synchronized EOF. */
        let ghost_space = scroll.mapper.preview_ghost_space();
        if ghost_space > 0.0 {
            ui.add_space(ghost_space);
        }
        const PREVIEW_PANE_TOP_BOTTOM_PADDING: f32 = 4.0;
        ui.add_space(PREVIEW_PANE_TOP_BOTTOM_PADDING);
        const SCROLL_PAST_END_RATIO: f32 = 0.9;
        let viewport_pad = ui.clip_rect().height() * SCROLL_PAST_END_RATIO;
        ui.add_space(viewport_pad);
    }

    pub fn render_floating_buttons(
        ui: &mut egui::Ui,
        has_doc: bool,
        show_back_to_top: bool,
        _action: &mut AppAction,
        preview: &mut crate::preview_pane::PreviewPane,
    ) {
        let mut button_count = 0.0;
        if has_doc {
            button_count += 2.0;
        }
        if show_back_to_top {
            button_count += 1.0;
        }

        if button_count > 0.0 {
            const BUTTON_ROUNDING: f32 = 4.0;
            const BUTTON_MARGIN: f32 = 20.0;
            const BUTTON_SIZE: f32 = 32.0;
            const BACK_TO_TOP_LEFT_OFFSET: f32 = 5.0;

            let margin = BUTTON_MARGIN;
            let btn_size = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);
            let spacing = ui.spacing().item_spacing.x;
            let total_width = (btn_size.x * button_count) + (spacing * (button_count - 1.0));

            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(
                    ui.max_rect().right() - margin - total_width - BACK_TO_TOP_LEFT_OFFSET,
                    ui.max_rect().bottom() - margin - btn_size.y,
                ),
                egui::vec2(total_width, btn_size.y),
            );

            let mut overlay_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(btn_rect)
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
            );

            let icon_bg = if ui.visuals().dark_mode {
                crate::theme_bridge::TRANSPARENT
            } else {
                crate::theme_bridge::ThemeBridgeOps::from_gray(crate::shell_ui::LIGHT_MODE_ICON_BG)
            };

            overlay_ui.scope(|ui| {
                ui.visuals_mut().widgets.inactive.bg_fill = icon_bg;

                if show_back_to_top {
                    let btn = egui::Button::image(
                        crate::Icon::ArrowUp.ui_image(ui, crate::icon::IconSize::Medium),
                    )
                    .rounding(egui::Rounding::same(BUTTON_ROUNDING as u8))
                    .fill(icon_bg);
                    if ui
                        .add(btn)
                        .on_hover_text(crate::i18n::I18nOps::get().action.back_to_top.clone())
                        .clicked()
                    {
                        preview.scroll_request = Some(0);
                    }
                }
            });
        }
    }
}
