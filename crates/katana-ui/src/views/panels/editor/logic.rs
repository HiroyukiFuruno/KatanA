use super::types::*;
use crate::app_state::{AppAction, ScrollSource};
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

    pub fn render_context_menu(
        _ui: &mut egui::Ui,
        response: &egui::Response,
        action: &mut AppAction,
    ) {
        response.context_menu(|ui| {
            if ui
                .button(crate::i18n::I18nOps::get().action.save.as_str())
                .clicked()
            {
                *action = AppAction::SaveDocument;
                ui.close_menu();
            }
            if ui
                .button(
                    crate::i18n::I18nOps::get()
                        .search
                        .command_ingest_clipboard_image
                        .as_str(),
                )
                .clicked()
            {
                *action = AppAction::IngestClipboardImage;
                ui.close_menu();
            }
            if ui
                .button(
                    crate::i18n::I18nOps::get()
                        .search
                        .command_ingest_image_file
                        .as_str(),
                )
                .clicked()
            {
                *action = AppAction::IngestImageFile;
                ui.close_menu();
            }
        });
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
                let mut rect = egui::Rect::from_min_max(
                    egui::pos2(response.rect.min.x, response.rect.min.y + pos.min.y),
                    egui::pos2(response.rect.max.x, response.rect.min.y + pos.max.y),
                );
                const TOC_NAV_VERTICAL_OFFSET: f32 = 5.0;
                rect.min.y -= TOC_NAV_VERTICAL_OFFSET;
                ui.scroll_to_rect(rect, Some(egui::Align::TOP));
            }
        }
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
