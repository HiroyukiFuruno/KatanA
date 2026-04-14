use super::types::*;
use crate::app_state::ScrollSource;
use eframe::egui;

impl EditorLogicOps {
    /// Count the line (paragraph) number for a given character index in the buffer.
    pub fn char_index_to_line(buffer: &str, char_idx: usize) -> usize {
        buffer
            .chars()
            .take(char_idx)
            .filter(|&ch| ch == '\n')
            .count()
    }

    /// Convert a line number to the character index at the start of that line.
    pub fn line_to_char_index(buffer: &str, target_line: usize) -> Option<usize> {
        let mut current_line = 0;
        for (char_idx, c) in buffer.chars().enumerate() {
            if current_line == target_line {
                return Some(char_idx);
            }
            if c == '\n' {
                current_line += 1;
            }
        }
        None
    }

    /// Apply a pending cursor to a TextEdit widget after an authoring transform.
    /// Pattern: TextEdit::load_state → set_char_range → TextEdit::store_state
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
            /* WHY:
             * For editor, last_scroll_to_line is managed when navigation occurs.
             * Wait, we SHOULD update last_scroll_to_line so we don't jump every frame
             * but `process_helpers.rs` clears last_scroll_to_line, so we only jump once per navigation.
             */
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

    /// Convert a line range to (start_char_index, end_char_index) in the buffer.
    pub fn line_range_to_char_range(
        buffer: &str,
        line_start: usize,
        line_end: usize,
    ) -> Option<(usize, usize)> {
        let mut current_line = 0;
        let mut start_char = None;
        let mut end_char = None;

        for (char_idx, c) in buffer.chars().enumerate() {
            if current_line == line_start && start_char.is_none() {
                start_char = Some(char_idx);
            }
            if current_line == line_end + 1 {
                end_char = Some(char_idx.saturating_sub(1));
                break;
            }
            if c == '\n' {
                current_line += 1;
            }
        }
        if start_char.is_some() && end_char.is_none() {
            end_char = Some(buffer.chars().count().saturating_sub(1));
        }

        match (start_char, end_char) {
            (Some(s), Some(e)) => Some((s, e)),
            _ => None,
        }
    }

    /// Update scroll synchronization state after editor rendering.
    pub fn update_scroll_sync(
        scroll: &mut crate::app_state::ScrollState,
        content_height: f32,
        inner_rect_height: f32,
        current_offset_y: f32,
        was_consuming_preview: bool,
        _dead_zone: f32,
        anchors: Vec<f32>,
    ) {
        let max_scroll = (content_height - inner_rect_height).max(0.0);
        scroll.editor_max = max_scroll;
        scroll.editor_y = current_offset_y;

        if !anchors.is_empty() {
            scroll.editor_line_anchors = anchors;
        }

        if was_consuming_preview {
            scroll.source = ScrollSource::Neither;
            scroll.editor_echo.record(current_offset_y);
            return;
        }

        if scroll.source == ScrollSource::Editor {
            scroll.editor_echo.record(current_offset_y);
        }

        if max_scroll <= 0.0 {
            return;
        }

        /* WHY: Did the editor actually scroll from user interaction? */
        if scroll.editor_echo.is_echo(current_offset_y) {
            return;
        }

        /* WHY: When scroll_to_line is active (e.g. TOC navigation), the editor */
        /* WHY: was scrolled programmatically, not by user interaction. The preview */
        /* WHY: pane has its own scroll_request for direct heading navigation.     */
        /* WHY: Emitting ScrollSource::Editor here would cause compute_forced_offset */
        /* WHY: to overwrite the preview's position via mapper approximation,       */
        /* WHY: creating a click-highlight misalignment in the TOC.                */
        if scroll.scroll_to_line.is_some() {
            scroll.editor_echo.record(current_offset_y);
            return;
        }

        /* WHY: Recompute our logical position from the current pixel offset, */
        /* WHY: using the shared mapper built by Preview pane. */
        let next_logical = scroll.mapper.editor_to_logical(current_offset_y);

        /* WHY: We don't have a reliable 'dead_zone' based on float difference anymore, */
        /* WHY: because progress is not global. But pixels changed beyond ECHO_PIXEL_EPSILON. */
        if next_logical != scroll.logical_position {
            scroll.logical_position = next_logical;
            scroll.source = ScrollSource::Editor;
        }
    }

    /// Renders the context menu for the given response.
    pub fn render_context_menu(
        response: &egui::Response,
        action: &mut crate::app_state::AppAction,
    ) {
        response.context_menu(|ui| {
            let i18n = crate::i18n::I18nOps::get();
            if ui
                .button(&i18n.search.command_ingest_clipboard_image)
                .clicked()
            {
                *action = crate::app_state::AppAction::IngestClipboardImage;
                ui.close_menu();
            }
            if ui.button(&i18n.search.command_ingest_image_file).clicked() {
                *action = crate::app_state::AppAction::IngestImageFile;
                ui.close_menu();
            }
        });
    }
}

#[cfg(test)]
include!("logic_tests.rs");
