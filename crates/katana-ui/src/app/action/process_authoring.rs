use crate::app::*;
use crate::app_action::MarkdownAuthoringOp;
use crate::shell::*;
use crate::views::panels::editor::authoring::MarkdownAuthoringOps;

impl KatanaApp {
    /// Apply a Markdown authoring operation to the active document buffer.
    ///
    /// The `egui::TextEdit` stores its cursor state in `egui::Id`-keyed memory.
    /// We read the current cursor range via `egui::TextEdit::load_with_output` — if
    /// that is unavailable (palette call with no editor focused), we apply at the end.
    pub(crate) fn handle_action_author_markdown(&mut self, op: MarkdownAuthoringOp) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        if doc.is_reference {
            return;
        }
        let buffer = doc.buffer.clone();

        /* WHY: Retrieve the stored cursor range from the TextEdit widget's egui memory.
        The editor stores its cursor under egui::Id::new("katana_editor_cursor_range"). */
        let (sel_start, sel_end) = self
            .editor_cursor_range
            .map(|r| {
                let lo = r.primary.index.min(r.secondary.index);
                let hi = r.primary.index.max(r.secondary.index);
                /* WHY: CCursor indices are char-counts; convert to byte offsets. */
                let lo_byte = buffer
                    .char_indices()
                    .nth(lo)
                    .map(|(i, _)| i)
                    .unwrap_or(buffer.len());
                let hi_byte = buffer
                    .char_indices()
                    .nth(hi)
                    .map(|(i, _)| i)
                    .unwrap_or(buffer.len());
                (lo_byte, hi_byte)
            })
            .unwrap_or((buffer.len(), buffer.len()));

        let transform = MarkdownAuthoringOps::apply(&buffer, sel_start, sel_end, op);

        /* WHY: Commit the transformed buffer via the same path as normal typing,
        which preserves dirty-buffer detection and preview sync. */
        self.handle_update_buffer(transform.buffer.clone());

        /* WHY: Store the result cursor range so the editor can restore the selection
        on the next frame (see EditorContent::show). */
        let new_char_start = transform.buffer[..transform.cursor_start].chars().count();
        let new_char_end = transform.buffer[..transform.cursor_end].chars().count();
        self.pending_editor_cursor = Some((new_char_start, new_char_end));
    }

    /// Stub for image file ingest — implemented in Task 2.
    pub(crate) fn handle_action_ingest_image_file(&mut self) {
        tracing::debug!("IngestImageFile: not yet implemented (Task 2)");
    }

    /// Stub for clipboard image ingest — implemented in Task 2.
    pub(crate) fn handle_action_ingest_clipboard_image(&mut self) {
        tracing::debug!("IngestClipboardImage: not yet implemented (Task 2)");
    }

    /// Reveal an image asset path in the OS file manager.
    /// Implemented here because it reuses the existing RevealInOs infrastructure.
    pub(crate) fn handle_action_reveal_image_asset(&mut self, path: std::path::PathBuf) {
        self.handle_action_reveal_in_os(path);
    }
}
