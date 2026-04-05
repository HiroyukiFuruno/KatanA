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

                        let current_cursor_y =
                            super::decorations::EditorDecorations::render_cursor_line(
                                ui,
                                &buffer,
                                &galley,
                                text_output.cursor_range,
                                scroll,
                                &ln_rect,
                                &response.rect,
                                current_line_bg,
                            );

                        super::decorations::EditorDecorations::render_hovered_lines(
                            ui,
                            &buffer,
                            &galley,
                            scroll,
                            &ln_rect,
                            &response.rect,
                            hover_line_bg,
                        );

                        super::decorations::EditorDecorations::render_search_matches(
                            ui,
                            &galley,
                            &response.rect,
                            self.doc_search_matches,
                            self.doc_search_active_index,
                        );

                        super::line_numbers::EditorLineNumbers::render(
                            ui,
                            &galley,
                            &response.rect,
                            &ln_rect,
                            scroll,
                            current_cursor_y,
                            ln_text,
                            ln_active_text,
                            left_margin,
                            LINE_NUMBER_PAD_RIGHT,
                        );

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
