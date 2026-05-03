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
        workspace_root: Option<&std::path::Path>,
        scroll: &mut crate::app_state::ScrollState,
        action: &mut AppAction,
        sync_scroll: bool,
        doc_search_matches: &[std::ops::Range<usize>],
        doc_search_active_index: usize,
        cursor_range_out: &mut Option<egui::text::CCursorRange>,
        pending_cursor: Option<(usize, usize)>,
        diagnostics: &[katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
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
            let editor_id =
                crate::editor_undo::EditorUndoIdentity::text_edit_id(workspace_root, &doc.path);
            let response_had_focus_before_edit =
                ui.memory(|mem| mem.focused().is_some_and(|id| id == editor_id));
            let clipboard_image_paste_intercepted =
                super::paste::EditorPasteOps::intercept_clipboard_image_paste(
                    ui.ctx(),
                    response_had_focus_before_edit,
                    crate::app::action::clipboard_image::ClipboardImageOps::has_image_payload,
                );
            let events_before_edit = ui.input(|input| input.events.clone());

            let text_edit = egui::TextEdit::multiline(buffer)
                .id(editor_id)
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

            let previous_cursor_range = *cursor_range_out;
            let text_output = text_edit.show(ui);
            let response = text_output.response;

            super::context_menu::EditorContextMenu::render(
                &response,
                action,
                &doc.path,
                editable,
                text_output.cursor_range,
            );
            let galley = text_output.galley;

            let popup_cursor_range = text_output.cursor_range.or(*cursor_range_out);
            if let Some(range) = text_output.cursor_range {
                *cursor_range_out = Some(range);
            }

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

            let diagnostic_hovered = super::diagnostics_ui::EditorDiagnostics::render_diagnostics(
                ui,
                buffer,
                &galley,
                &response.rect,
                diagnostics,
                action,
            );

            const PAD_RIGHT: f32 = 8.0;
            let line_diagnostic_hovered = super::line_numbers::EditorLineNumbers::render(
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

            ToolbarPopup::show(
                ui,
                action,
                &response,
                &galley,
                popup_cursor_range,
                editable,
                diagnostic_hovered || line_diagnostic_hovered,
            );

            let text_changed = response.changed();
            let response_has_focus_after_edit = response.has_focus()
                || ui.memory(|mem| mem.focused().is_some_and(|id| id == response.id));
            let should_ingest_clipboard_image = ui.input(|input| {
                EditorLogicOps::editor_clipboard_image_paste_requested_from_event_snapshots(
                    response_had_focus_before_edit,
                    response_has_focus_after_edit,
                    text_changed,
                    &events_before_edit,
                    &input.events,
                )
            });
            if editable && clipboard_image_paste_intercepted {
                *action = AppAction::IngestClipboardImage;
            } else if editable && should_ingest_clipboard_image {
                if text_changed {
                    *cursor_range_out = previous_cursor_range;
                }
                *action = AppAction::IngestClipboardImage;
            } else if text_changed {
                *action = AppAction::UpdateBuffer(buffer.clone());
            }
            EditorLogicOps::handle_scroll_to_line(ui, scroll, buffer, &response, &galley);
            let anchors = EditorLogicOps::extract_line_anchors(&galley);
            (response, anchors)
        })
    }
}
