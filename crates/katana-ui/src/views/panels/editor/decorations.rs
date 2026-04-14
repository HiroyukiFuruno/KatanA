use crate::views::panels::editor::types::EditorLogicOps;
use eframe::egui;

pub(crate) struct EditorDecorations;

pub(crate) struct CursorLineParams<'a> {
    pub buffer: &'a str,
    pub galley: &'a std::sync::Arc<egui::Galley>,
    pub cursor_range: Option<egui::text::CCursorRange>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub ln_rect: &'a egui::Rect,
    pub response_rect: &'a egui::Rect,
    pub current_line_bg: Option<egui::Color32>,
}

impl EditorDecorations {
    pub(crate) fn render_cursor_line(
        ui: &mut egui::Ui,
        params: CursorLineParams<'_>,
    ) -> Option<f32> {
        let CursorLineParams {
            buffer,
            galley,
            cursor_range,
            scroll,
            ln_rect,
            response_rect,
            current_line_bg,
        } = params;
        let mut current_cursor_y = None;
        if let Some(c) = cursor_range {
            let paragraph = EditorLogicOps::char_index_to_line(buffer, c.primary.index);
            scroll.active_editor_line = Some(paragraph);
            let cursor_rect = galley.pos_from_cursor(c.primary);
            current_cursor_y = Some(cursor_rect.min.y);
            let highlight_rect = egui::Rect::from_min_max(
                egui::pos2(ln_rect.min.x, response_rect.min.y + cursor_rect.min.y),
                egui::pos2(response_rect.max.x, response_rect.min.y + cursor_rect.max.y),
            );
            let highlight_color = EditorLogicOps::current_line_highlight_color(
                ui.visuals().dark_mode,
                current_line_bg,
            );
            ui.painter()
                .rect_filled(highlight_rect, 1.0, highlight_color);
        } else {
            scroll.active_editor_line = None;
        }
        current_cursor_y
    }

    pub(crate) fn render_hovered_lines(
        ui: &mut egui::Ui,
        buffer: &str,
        galley: &std::sync::Arc<egui::Galley>,
        scroll: &crate::app_state::ScrollState,
        ln_rect: &egui::Rect,
        response_rect: &egui::Rect,
        hover_line_bg: Option<egui::Color32>,
    ) {
        let hover_color =
            EditorLogicOps::hover_line_highlight_color(ui.visuals().dark_mode, hover_line_bg);
        for line_range in &scroll.hovered_preview_lines {
            let Some((start_idx, end_idx)) =
                EditorLogicOps::line_range_to_char_range(buffer, line_range.start, line_range.end)
            else {
                continue;
            };
            let cursor_start = egui::text::CCursor {
                index: start_idx,
                prefer_next_row: false,
            };
            let cursor_end = egui::text::CCursor {
                index: end_idx.saturating_sub(1),
                prefer_next_row: false,
            };
            let pos_start = galley.pos_from_cursor(cursor_start);
            let pos_end = galley.pos_from_cursor(cursor_end);
            let highlight_rect = egui::Rect::from_min_max(
                egui::pos2(ln_rect.min.x, response_rect.min.y + pos_start.min.y),
                egui::pos2(response_rect.max.x, response_rect.min.y + pos_end.max.y),
            );
            ui.painter().rect_filled(highlight_rect, 1.0, hover_color);
        }
    }

    pub(crate) fn render_search_matches(
        ui: &mut egui::Ui,
        galley: &std::sync::Arc<egui::Galley>,
        response_rect: &egui::Rect,
        doc_search_matches: &[std::ops::Range<usize>],
        doc_search_active_index: usize,
    ) {
        let (search_match_color, search_active_color) = ui.ctx().data(|d| {
            let tc = d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                "katana_theme_colors",
            ));
            if let Some(theme) = tc {
                (
                    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(theme.code.search_match),
                    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(theme.code.search_active),
                )
            } else {
                (ui.visuals().selection.bg_fill, ui.visuals().warn_fg_color)
            }
        });
        for (idx, range) in doc_search_matches.iter().enumerate() {
            let match_start = egui::text::CCursor {
                index: range.start,
                prefer_next_row: false,
            };
            let match_end = egui::text::CCursor {
                index: range.end,
                prefer_next_row: false,
            };
            let color = if idx == doc_search_active_index {
                search_active_color
            } else {
                search_match_color
            };

            let start_row = galley.layout_from_cursor(match_start).row;
            let end_row = galley.layout_from_cursor(match_end).row;

            Self::paint_match(
                ui,
                &SearchMatchCtx {
                    galley,
                    response_rect,
                    match_start,
                    match_end,
                    start_row,
                    end_row,
                    color,
                },
            );
        }
    }

    fn paint_match(ui: &mut egui::Ui, ctx: &SearchMatchCtx) {
        for row_idx in ctx.start_row..=ctx.end_row {
            let Some(placed_row) = ctx.galley.rows.get(row_idx) else {
                continue;
            };
            let row_rect = placed_row.rect();
            let (left_x, right_x) = if row_idx == ctx.start_row {
                let pos_start = ctx.galley.pos_from_cursor(ctx.match_start);
                (
                    pos_start.min.x.max(0.0),
                    if ctx.start_row == ctx.end_row {
                        pos_start.min.x
                    } else {
                        row_rect.right()
                    },
                )
            } else if row_idx == ctx.end_row {
                let pos_end = ctx.galley.pos_from_cursor(ctx.match_end);
                (row_rect.left().max(0.0), pos_end.max.x.max(0.0))
            } else {
                (row_rect.left().max(0.0), row_rect.right())
            };

            /* WHY: For single-row matches, use start/end cursor positions directly. */
            let right_x = if ctx.start_row == ctx.end_row {
                let pos_end = ctx.galley.pos_from_cursor(ctx.match_end);
                pos_end.max.x.max(0.0)
            } else {
                right_x
            };

            let rect = egui::Rect::from_min_max(
                egui::pos2(
                    ctx.response_rect.min.x + left_x,
                    ctx.response_rect.min.y + row_rect.top(),
                ),
                egui::pos2(
                    ctx.response_rect.min.x + right_x,
                    ctx.response_rect.min.y + row_rect.bottom(),
                ),
            );
            if rect.is_positive() {
                ui.painter().rect_filled(rect, 2.0, ctx.color);
            }
        }
    }
}

struct SearchMatchCtx<'a> {
    galley: &'a std::sync::Arc<egui::Galley>,
    response_rect: &'a egui::Rect,
    match_start: egui::text::CCursor,
    match_end: egui::text::CCursor,
    start_row: usize,
    end_row: usize,
    color: egui::Color32,
}
