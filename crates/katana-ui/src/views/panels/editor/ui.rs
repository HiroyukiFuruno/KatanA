use crate::app_state::{AppAction, ScrollSource};
use crate::shell::SCROLL_SYNC_DEAD_ZONE;
use eframe::egui;

use super::types::{EditorColors, EditorLogicOps};
pub(crate) struct EditorContent<'a> {
    pub document: Option<&'a katana_core::document::Document>,
    pub workspace_root: Option<&'a std::path::Path>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub action: &'a mut AppAction,
    pub sync_scroll: bool,
    pub doc_search_matches: &'a [std::ops::Range<usize>],
    pub doc_search_active_index: usize,
    /// Output: receives the cursor range reported by `TextEdit` this frame.
    pub cursor_range_out: &'a mut Option<egui::text::CCursorRange>,
    /// Input: if set, the `TextEdit` cursor is programmatically moved to this
    /// char-index range on this frame (used after an authoring transform).
    pub pending_cursor: Option<(usize, usize)>,
    pub diagnostics: &'a [katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
}
impl<'a> EditorContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document: Option<&'a katana_core::document::Document>,
        workspace_root: Option<&'a std::path::Path>,
        scroll: &'a mut crate::app_state::ScrollState,
        action: &'a mut AppAction,
        sync_scroll: bool,
        doc_search_matches: &'a [std::ops::Range<usize>],
        doc_search_active_index: usize,
        cursor_range_out: &'a mut Option<egui::text::CCursorRange>,
        pending_cursor: Option<(usize, usize)>,
        diagnostics: &'a [katana_markdown_linter::rules::markdown::MarkdownDiagnostic],
    ) -> Self {
        Self {
            document,
            workspace_root,
            scroll,
            action,
            sync_scroll,
            doc_search_matches,
            doc_search_active_index,
            cursor_range_out,
            pending_cursor,
            diagnostics,
        }
    }
    pub fn show(self, ui: &mut egui::Ui) {
        let action = self.action;
        let sync_scroll = self.sync_scroll;
        let scroll = self.scroll;
        let cursor_range_out = self.cursor_range_out;
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
                    let horiz_response = super::text_edit::TextEditRenderer::render(
                        ui,
                        &mut buffer,
                        doc,
                        self.workspace_root,
                        scroll,
                        action,
                        sync_scroll,
                        self.doc_search_matches,
                        self.doc_search_active_index,
                        cursor_range_out,
                        self.pending_cursor,
                        self.diagnostics,
                        current_line_bg,
                        hover_line_bg,
                        ln_text,
                        ln_active_text,
                    );

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
