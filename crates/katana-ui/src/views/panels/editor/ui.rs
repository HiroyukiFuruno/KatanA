use crate::app_state::{AppAction, ScrollSource};
use crate::shell::{EDITOR_INITIAL_VISIBLE_ROWS, SCROLL_SYNC_DEAD_ZONE};
use eframe::egui;

use super::types::{EditorColors, EditorLogicOps};

pub(crate) struct EditorContent<'a> {
    pub document: Option<&'a katana_core::document::Document>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub action: &'a mut AppAction,
    pub sync_scroll: bool,
    pub doc_search_matches: &'a [std::ops::Range<usize>],
    pub doc_search_active_index: usize,
}

impl<'a> EditorContent<'a> {
    pub fn new(
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        action: &'a mut AppAction,
        sync_scroll: bool,
        doc_search_matches: &'a [std::ops::Range<usize>],
        doc_search_active_index: usize,
    ) -> Self {
        Self {
            document,
            scroll,
            action,
            sync_scroll,
            doc_search_matches,
            doc_search_active_index,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let action = self.action;
        let sync_scroll = self.sync_scroll;
        let scroll = self.scroll;
        if let Some(doc) = self.document {
            let mut buffer = doc.buffer.clone();

            let colors: EditorColors = EditorLogicOps::resolve_editor_colors(ui);
            let (
                code_bg,
                code_text,
                code_selection,
                current_line_bg,
                hover_line_bg,
                ln_text,
                ln_active_text,
            ) = colors;

            let mut scroll_area = egui::ScrollArea::vertical().id_salt("editor_scroll");

            let consuming_preview = sync_scroll && scroll.source == ScrollSource::Preview;
            if consuming_preview {
                scroll_area = scroll_area.vertical_scroll_offset(
                    scroll.mapper.logical_to_editor(scroll.logical_position),
                );
            }

            let output = egui::Frame::NONE.fill(code_bg).show(ui, |ui| {
                ui.style_mut().visuals.override_text_color = Some(code_text);
                ui.style_mut().visuals.extreme_bg_color = code_bg;
                if let Some(sel) = code_selection {
                    ui.style_mut().visuals.selection.bg_fill = sel;
                }

                scroll_area.show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        const LINE_NUMBER_MARGIN: f32 = 40.0;
                        const LINE_NUMBER_PAD_RIGHT: f32 = 8.0;
                        let left_margin = LINE_NUMBER_MARGIN;
                        let (ln_rect, _) = ui.allocate_exact_size(
                            egui::vec2(left_margin, 0.0),
                            egui::Sense::hover(),
                        );
                        let text_output = egui::TextEdit::multiline(&mut buffer)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(EDITOR_INITIAL_VISIBLE_ROWS)
                            .margin(egui::Margin {
                                left: 0,
                                right: LINE_NUMBER_MARGIN as i8,
                                top: 0,
                                bottom: 0,
                            })
                            .frame(egui::Frame::NONE)
                            .show(ui);
                        let response = text_output.response;
                        let galley = text_output.galley;

                        if response.clicked()
                            && let Some(c) = text_output.cursor_range
                        {
                            let line = EditorLogicOps::char_index_to_line(&buffer, c.primary.index);
                            scroll.scroll_to_line = Some(line);
                        }

                        let mut current_cursor_y = None;
                        if let Some(c) = text_output.cursor_range {
                            let paragraph =
                                EditorLogicOps::char_index_to_line(&buffer, c.primary.index);
                            scroll.active_editor_line = Some(paragraph);

                            let cursor_rect = galley.pos_from_cursor(c.primary);
                            current_cursor_y = Some(cursor_rect.min.y);

                            let highlight_rect = egui::Rect::from_min_max(
                                egui::pos2(ln_rect.min.x, response.rect.min.y + cursor_rect.min.y),
                                egui::pos2(
                                    response.rect.max.x,
                                    response.rect.min.y + cursor_rect.max.y,
                                ),
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

                        let hover_color = EditorLogicOps::hover_line_highlight_color(
                            ui.visuals().dark_mode,
                            hover_line_bg,
                        );

                        for line_range in &scroll.hovered_preview_lines {
                            if let Some((start_idx, end_idx)) =
                                EditorLogicOps::line_range_to_char_range(
                                    &buffer,
                                    line_range.start,
                                    line_range.end,
                                )
                            {
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
                                    egui::pos2(
                                        ln_rect.min.x,
                                        response.rect.min.y + pos_start.min.y,
                                    ),
                                    egui::pos2(
                                        response.rect.max.x,
                                        response.rect.min.y + pos_end.max.y,
                                    ),
                                );
                                ui.painter().rect_filled(highlight_rect, 1.0, hover_color);
                            }
                        }

                        let (search_match_color, search_active_color) = ui.ctx().data(|d| {
                            let tc = d.get_temp::<katana_platform::theme::ThemeColors>(
                                egui::Id::new("katana_theme_colors"),
                            );
                            if let Some(theme) = tc {
                                (
                                    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                                        theme.code.search_match,
                                    ),
                                    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                                        theme.code.search_active,
                                    ),
                                )
                            } else {
                                (ui.visuals().selection.bg_fill, ui.visuals().warn_fg_color)
                            }
                        });
                        for (idx, range) in self.doc_search_matches.iter().enumerate() {
                            let match_start = egui::text::CCursor {
                                index: range.start,
                                prefer_next_row: false,
                            };
                            let match_end = egui::text::CCursor {
                                index: range.end,
                                prefer_next_row: false,
                            };
                            let pos_start = galley.pos_from_cursor(match_start);
                            let pos_end = galley.pos_from_cursor(match_end);

                            let highlight_rect = egui::Rect::from_min_max(
                                egui::pos2(
                                    response.rect.min.x + pos_start.min.x.max(0.0),
                                    response.rect.min.y + pos_start.min.y,
                                ),
                                egui::pos2(
                                    response.rect.min.x + pos_end.min.x.max(0.0), /* WHY: using min.x for both, since it's cursor position. Wait, pos_end's cursor is BEFORE the text? Actually wait, CCursor index. */
                                    response.rect.min.y + pos_end.max.y,
                                ),
                            );

                            let color = if idx == self.doc_search_active_index {
                                search_active_color
                            } else {
                                search_match_color
                            };
                            ui.painter().rect_filled(highlight_rect, 2.0, color);
                        }

                        let clip_rect = ui.clip_rect().expand(100.0);
                        let mut p = 0;
                        let mut is_start_of_para = true;

                        for row in &galley.rows {
                            let top_y = row.rect().min.y;
                            let y = response.rect.min.y + top_y;
                            let is_visible = is_start_of_para
                                && y <= clip_rect.max.y
                                && (y + row.rect().height()) >= clip_rect.min.y;

                            if is_visible {
                                let is_current = current_cursor_y == Some(top_y);
                                let text = format!("{}", p + 1);
                                let color: egui::Color32 = if is_current {
                                    ln_active_text.unwrap_or_else(|| -> egui::Color32 {
                                        ui.visuals().text_color()
                                    })
                                } else {
                                    const LINE_NUMBER_INACTIVE_ALPHA: f32 = 0.3;
                                    ln_text.unwrap_or_else(|| -> egui::Color32 {
                                        ui.visuals()
                                            .text_color()
                                            .linear_multiply(LINE_NUMBER_INACTIVE_ALPHA)
                                    })
                                };
                                let font_id = egui::TextStyle::Monospace.resolve(ui.style());

                                let label_rect = egui::Rect::from_min_size(
                                    egui::pos2(ln_rect.min.x, y),
                                    egui::vec2(
                                        left_margin - LINE_NUMBER_PAD_RIGHT,
                                        row.rect().height(),
                                    ),
                                );
                                let mut text_rt =
                                    egui::RichText::new(text).color(color).font(font_id);
                                if is_current {
                                    text_rt = text_rt.strong();
                                }

                                let label_for_measuring =
                                    egui::Label::new(text_rt.clone()).selectable(false);
                                let galley_ln = label_for_measuring.layout_in_ui(ui);
                                let offset_x =
                                    (label_rect.width() - galley_ln.1.rect.width()).max(0.0);
                                let tight_rect = egui::Rect::from_min_size(
                                    label_rect.min + egui::vec2(offset_x, 0.0),
                                    galley_ln.1.rect.size(),
                                );

                                let resp =
                                    ui.interact(label_rect, ui.id().with(p), egui::Sense::click());
                                if resp.clicked() {
                                    scroll.scroll_to_line = Some(p);
                                }
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                ui.put(tight_rect, egui::Label::new(text_rt).selectable(false));
                            }

                            is_start_of_para = row.ends_with_newline;
                            if row.ends_with_newline {
                                p += 1;
                            }
                        }

                        if response.changed() {
                            *action = AppAction::UpdateBuffer(buffer.clone());
                        }

                        if let Some(target_line) = scroll.scroll_to_line
                            && let Some(idx) =
                                EditorLogicOps::line_to_char_index(&buffer, target_line)
                        {
                            let cursor = egui::text::CCursor {
                                index: idx,
                                prefer_next_row: false,
                            };
                            let pos = galley.pos_from_cursor(cursor);
                            let rect = egui::Rect::from_min_max(
                                egui::pos2(response.rect.min.x, response.rect.min.y + pos.min.y),
                                egui::pos2(response.rect.max.x, response.rect.min.y + pos.max.y),
                            );
                            ui.scroll_to_rect(rect, Some(egui::Align::Center));
                        }
                        response
                    })
                    .inner
                })
            });

            if sync_scroll {
                EditorLogicOps::update_scroll_sync(
                    scroll,
                    output.inner.content_size.y,
                    output.inner.inner_rect.height(),
                    output.inner.state.offset.y,
                    consuming_preview,
                    SCROLL_SYNC_DEAD_ZONE,
                );
            }
        }
    }
}
