use super::types::*;
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
                    index: egui::text::CharIndex(char_end),
                    prefer_next_row: false,
                },
                secondary: egui::text::CCursor {
                    index: egui::text::CharIndex(char_start),
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

    #[cfg(test)]
    pub fn intercept_clipboard_image_paste_for_test<F>(
        ctx: &egui::Context,
        response_has_focus: bool,
        clipboard_has_image: F,
    ) -> bool
    where
        F: FnOnce() -> bool,
    {
        super::paste::EditorPasteOps::intercept_clipboard_image_paste(
            ctx,
            response_has_focus,
            clipboard_has_image,
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

    pub fn editor_clipboard_image_paste_requested_from_event_snapshots(
        response_had_focus_before_edit: bool,
        response_has_focus_after_edit: bool,
        text_changed: bool,
        events_before_edit: &[egui::Event],
        events_after_edit: &[egui::Event],
    ) -> bool {
        Self::should_ingest_clipboard_image_paste(
            response_had_focus_before_edit || response_has_focus_after_edit,
            text_changed,
            events_before_edit,
        ) || Self::should_ingest_clipboard_image_paste(
            response_has_focus_after_edit,
            text_changed,
            events_after_edit,
        )
    }
}

#[cfg(test)]
include!("logic_tests.rs");
