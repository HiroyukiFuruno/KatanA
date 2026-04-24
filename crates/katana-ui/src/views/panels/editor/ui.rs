use crate::app_state::{AppAction, ScrollSource};
use crate::shell::{EDITOR_INITIAL_VISIBLE_ROWS, SCROLL_SYNC_DEAD_ZONE};
use eframe::egui;

use super::toolbar_popup::ToolbarPopup;
use super::types::{EditorColors, EditorLogicOps};
pub(crate) struct EditorContent<'a> {
    pub document: Option<&'a katana_core::document::Document>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub action: &'a mut AppAction,
    pub sync_scroll: bool,
    pub doc_search_matches: &'a [std::ops::Range<usize>],
    pub doc_search_active_index: usize,
    pub cursor_range_out: &'a mut Option<egui::text::CCursorRange>,
    pub pending_cursor: Option<(usize, usize)>,
}
impl<'a> EditorContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut crate::app_state::ScrollState,
        action: &'a mut AppAction,
        sync_scroll: bool,
        doc_search_matches: &'a [std::ops::Range<usize>],
        doc_search_active_index: usize,
        cursor_range_out: &'a mut Option<egui::text::CCursorRange>,
        pending_cursor: Option<(usize, usize)>,
    ) -> Self {
        Self {
            document,
            scroll,
            action,
            sync_scroll,
            doc_search_matches,
            doc_search_active_index,
            cursor_range_out,
            pending_cursor,
        }
    }
    pub fn show(self, ui: &mut egui::Ui) {
        let action = self.action;
        let sync_scroll = self.sync_scroll;
        let scroll = self.scroll;
        let cursor_range_out = self.cursor_range_out;
        if let Some(doc) = self.document {
            let mut buffer = doc.buffer.clone();
            let editable = !doc.is_reference;
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
            let mut scroll_area =
                egui::ScrollArea::vertical().id_salt(("editor_scroll", &doc.path));
            if scroll.scroll_to_line.is_some() {
                scroll_area = scroll_area.animated(false);
            }
            let consuming_preview = sync_scroll
                && scroll.source == ScrollSource::Preview
                && scroll.scroll_to_line.is_none();
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
                    let horiz_response = ui.horizontal_top(|ui| {
                        const LINE_NUMBER_MARGIN: f32 = 40.0;
                        let (ln_rect, _) = ui.allocate_exact_size(
                            egui::vec2(LINE_NUMBER_MARGIN, 0.0),
                            egui::Sense::hover(),
                        );
                        let text_edit = egui::TextEdit::multiline(&mut buffer)
                            .id(egui::Id::new("editor_text_edit"))
                            .interactive(editable)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(EDITOR_INITIAL_VISIBLE_ROWS)
                            .margin(egui::Margin {
                                left: 0,
                                right: LINE_NUMBER_MARGIN as i8,
                                top: 0,
                                bottom: 0,
                            })
                            .frame(egui::Frame::NONE);
                        let text_output = text_edit.show(ui);
                        let response = text_output.response;
                        let galley = text_output.galley;
                        let cursor_range = text_output.cursor_range;
                        super::context_menu::EditorContextMenu::render(
                            &response,
                            action,
                            cursor_range,
                        );
                        if let Some(range) = cursor_range {
                            *cursor_range_out = Some(range);
                        }
                        ToolbarPopup::show(ui, action, &response, &galley, cursor_range, editable);
                        if let Some(cursor_range) = self.pending_cursor {
                            EditorLogicOps::apply_pending_cursor(ui, response.id, cursor_range);
                        }
                        if sync_scroll
                            && response.clicked()
                            && let Some(c) = cursor_range
                        {
                            let line = EditorLogicOps::char_index_to_line(&buffer, c.primary.index);
                            scroll.scroll_to_line = Some(line);
                        }
                        let current_cursor_y =
                            super::decorations::EditorDecorations::render_cursor_line(
                                ui,
                                super::decorations::CursorLineParams {
                                    buffer: &buffer,
                                    galley: &galley,
                                    cursor_range,
                                    scroll,
                                    ln_rect: &ln_rect,
                                    response_rect: &response.rect,
                                    current_line_bg,
                                },
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
                        const PAD_RIGHT: f32 = 8.0;
                        super::line_numbers::EditorLineNumbers::render(
                            ui,
                            super::line_numbers::LineNumberParams {
                                galley: &galley,
                                response_rect: &response.rect,
                                ln_rect: &ln_rect,
                                scroll,
                                current_cursor_y,
                                ln_text,
                                ln_active_text,
                                left_margin: LINE_NUMBER_MARGIN,
                                line_number_pad_right: PAD_RIGHT,
                            },
                        );

                        let text_changed = response.changed();
                        let should_ingest_clipboard_image =
                            EditorLogicOps::editor_clipboard_image_paste_requested(
                                ui,
                                &response,
                                text_changed,
                            );
                        if text_changed {
                            *action = AppAction::UpdateBuffer(buffer.clone());
                        } else if editable && should_ingest_clipboard_image {
                            *action = AppAction::IngestClipboardImage;
                        }
                        EditorLogicOps::handle_scroll_to_line(
                            ui, scroll, &buffer, &response, &galley,
                        );
                        let anchors = EditorLogicOps::extract_line_anchors(&galley);
                        (response, anchors)
                    });

                    EditorLogicOps::render_editor_padding(ui, scroll);
                    horiz_response
                })
            });
            let (_response, anchors) = output.inner.inner.inner;
            EditorLogicOps::update_scroll_sync(
                scroll,
                output.inner.content_size.y,
                output.inner.inner_rect.height(),
                output.inner.state.offset.y,
                consuming_preview,
                SCROLL_SYNC_DEAD_ZONE,
                anchors,
            );
        }
    }
}
