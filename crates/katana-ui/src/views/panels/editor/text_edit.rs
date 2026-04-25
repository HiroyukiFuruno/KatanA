use crate::app_state::AppAction;
use crate::shell::EDITOR_INITIAL_VISIBLE_ROWS;
use eframe::egui;

use super::toolbar_popup::ToolbarPopup;
use super::types::EditorLogicOps;

pub(crate) struct TextEditRenderer;

impl TextEditRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        ui: &mut egui::Ui,
        buffer: &mut String,
        doc: &katana_core::document::Document,
        scroll: &mut crate::app_state::ScrollState,
        action: &mut AppAction,
        sync_scroll: bool,
        doc_search_matches: &[std::ops::Range<usize>],
        doc_search_active_index: usize,
        cursor_range_out: &mut Option<egui::text::CCursorRange>,
        pending_cursor: Option<(usize, usize)>,
        diagnostics: &[katana_linter::rules::markdown::MarkdownDiagnostic],
        current_line_bg: Option<egui::Color32>,
        hover_line_bg: Option<egui::Color32>,
        ln_text: Option<egui::Color32>,
        ln_active_text: Option<egui::Color32>,
    ) -> egui::InnerResponse<(egui::AtomLayoutResponse, Vec<f32>)> {
        ui.horizontal_top(|ui| {
            /* WHY: Extra width accommodates the diagnostic gutter icon (14px) + margin without
             * overlapping the line numbers. Increased from 40 to 52. */
            const LINE_NUMBER_MARGIN: f32 = 52.0;
            let (ln_rect, _) =
                ui.allocate_exact_size(egui::vec2(LINE_NUMBER_MARGIN, 0.0), egui::Sense::hover());

            let editable = !doc.is_reference;
            let text_edit = egui::TextEdit::multiline(buffer)
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

            super::context_menu::EditorContextMenu::render(
                &response,
                action,
                text_output.cursor_range,
            );
            let galley = text_output.galley;

            let popup_cursor_range = text_output.cursor_range.or(*cursor_range_out);
            if let Some(range) = text_output.cursor_range {
                *cursor_range_out = Some(range);
            }
            ToolbarPopup::show(ui, action, &response, &galley, popup_cursor_range, editable);

            if let Some(cursor_range) = pending_cursor {
                EditorLogicOps::apply_pending_cursor(ui, response.id, cursor_range);
            }

            if sync_scroll
                && response.clicked()
                && let Some(c) = text_output.cursor_range
            {
                let line = EditorLogicOps::char_index_to_line(buffer, c.primary.index);
                scroll.scroll_to_line = Some(line);
            }

            let current_cursor_y = super::decorations::EditorDecorations::render_cursor_line(
                ui,
                super::decorations::CursorLineParams {
                    buffer,
                    galley: &galley,
                    cursor_range: text_output.cursor_range,
                    scroll,
                    ln_rect: &ln_rect,
                    response_rect: &response.rect,
                    current_line_bg,
                },
            );

            super::decorations::EditorDecorations::render_hovered_lines(
                ui,
                buffer,
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
                doc_search_matches,
                doc_search_active_index,
            );

            super::diagnostics_ui::EditorDiagnostics::render_diagnostics(
                ui,
                buffer,
                &galley,
                &response.rect,
                diagnostics,
                action,
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
                    diagnostics,
                    action,
                },
            );

            let text_changed = response.changed();
            let should_ingest_clipboard_image =
                EditorLogicOps::editor_clipboard_image_paste_requested(ui, &response, text_changed);
            if text_changed {
                *action = AppAction::UpdateBuffer(buffer.clone());
            } else if editable && should_ingest_clipboard_image {
                *action = AppAction::IngestClipboardImage;
            }
            EditorLogicOps::handle_scroll_to_line(ui, scroll, buffer, &response, &galley);
            let anchors = EditorLogicOps::extract_line_anchors(&galley);
            (response, anchors)
        })
    }
}
