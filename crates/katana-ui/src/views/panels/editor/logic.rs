use super::types::*;
use crate::app_state::ScrollSource;
use eframe::egui;

impl EditorLogicOps {
    /* WHY: Apply a pending cursor to a TextEdit widget after an authoring transform. */
    pub fn apply_pending_cursor(
        ui: &mut egui::Ui,
        editor_id: egui::Id,
        (char_start, char_end): (usize, usize),
    ) {
        if let Some(mut ts) = egui::TextEdit::load_state(ui.ctx(), editor_id) {
            let new_range = egui::text::CCursorRange {
                primary: egui::text::CCursor {
                    index: char_end,
                    prefer_next_row: false,
                },
                secondary: egui::text::CCursor {
                    index: char_start,
                    prefer_next_row: false,
                },
                h_pos: None,
            };
            ts.cursor.set_char_range(Some(new_range));
            egui::TextEdit::store_state(ui.ctx(), editor_id, ts);
        }
    }

    pub fn should_ingest_clipboard_image_paste(
        response_has_focus: bool,
        text_changed: bool,
        events: &[egui::Event],
    ) -> bool {
        super::paste::EditorPasteOps::should_ingest_clipboard_image_paste(
            response_has_focus,
            text_changed,
            events,
        )
    }

    pub fn editor_clipboard_image_paste_requested(
        ui: &egui::Ui,
        response: &egui::Response,
        text_changed: bool,
    ) -> bool {
        let response_has_focus = response.has_focus()
            || ui.memory(|mem| mem.focused().is_some_and(|id| id == response.id));
        ui.input(|i| {
            Self::should_ingest_clipboard_image_paste(response_has_focus, text_changed, &i.events)
        })
    }

    pub fn handle_scroll_to_line(
        ui: &mut egui::Ui,
        scroll: &mut crate::app_state::ScrollState,
        buffer: &str,
        response: &egui::Response,
        galley: &egui::text::Galley,
    ) {
        if let Some(target_line) = scroll.scroll_to_line {
            if scroll.last_scroll_to_line == Some(target_line) {
                return;
            }
            scroll.last_scroll_to_line = Some(target_line);

            if let Some(idx) = Self::line_to_char_index(buffer, target_line) {
                let cursor = egui::text::CCursor {
                    index: idx,
                    prefer_next_row: false,
                };
                let pos = galley.pos_from_cursor(cursor);
                let (rect, align) = Self::jump_target_scroll_rect(&response.rect, pos);
                ui.scroll_to_rect(rect, Some(align));
            }
        }
    }

    pub(crate) fn jump_target_scroll_rect(
        response_rect: &egui::Rect,
        cursor_rect: egui::Rect,
    ) -> (egui::Rect, egui::Align) {
        (
            egui::Rect::from_min_max(
                egui::pos2(response_rect.min.x, response_rect.min.y + cursor_rect.min.y),
                egui::pos2(response_rect.max.x, response_rect.min.y + cursor_rect.max.y),
            ),
            egui::Align::Center,
        )
    }

    pub fn update_scroll_sync(
        scroll: &mut crate::app_state::ScrollState,
        content_height: f32,
        inner_height: f32,
        offset_y: f32,
        consuming_preview: bool,
        dead_zone: f32,
        anchors: Vec<f32>,
    ) {
        let max_scroll = (content_height - inner_height).max(0.0);
        scroll.editor_max = max_scroll;
        scroll.editor_y = offset_y;
        scroll.editor_line_anchors = anchors;

        if consuming_preview {
            scroll.source = crate::app_state::ScrollSource::Neither;
            return;
        }

        if max_scroll <= 0.0 {
            return;
        }

        if scroll.editor_echo.is_echo(offset_y) {
            return;
        }

        if scroll.scroll_to_line.is_some() {
            return;
        }

        let current_logical = scroll.logical_position;
        let next_logical = scroll.mapper.editor_to_logical(offset_y);

        let dist = if next_logical.segment_index == current_logical.segment_index {
            (next_logical.progress - current_logical.progress).abs()
        } else {
            1.0
        };

        if dist > dead_zone {
            scroll.logical_position = next_logical;
            scroll.source = ScrollSource::Editor;
        }
    }
}

#[cfg(test)]
include!("logic_tests.rs");

#[cfg(test)]
#[path = "logic_scroll_tests.rs"]
mod logic_scroll_tests;
