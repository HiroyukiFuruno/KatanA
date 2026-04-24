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
        let safe_start = transform.cursor_start.min(transform.buffer.len());
        let safe_end = transform.cursor_end.min(transform.buffer.len());
        let new_char_start = transform.buffer[..safe_start].chars().count();
        let new_char_end = transform.buffer[..safe_end].chars().count();
        self.pending_editor_cursor = Some((new_char_start, new_char_end));
    }

    /// Helper to insert raw text into the document at cursor/selection
    pub(crate) fn handle_action_insert_raw_markdown(&mut self, text: &str) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        if doc.is_reference {
            return;
        }

        let buffer = doc.buffer.clone();
        let (sel_start, sel_end) = self
            .editor_cursor_range
            .map(|r| {
                let lo = r.primary.index.min(r.secondary.index);
                let hi = r.primary.index.max(r.secondary.index);
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

        let lo = sel_start.min(sel_end);
        let hi = sel_start.max(sel_end);

        /* WHY: Replace selection by combining prefix, snippet, suffix */
        let before = &buffer[..lo];
        let after = &buffer[hi..];
        let new_buffer = format!("{}{}{}", before, text, after);
        let new_cursor = before.len() + text.len();

        let new_char_cursor = new_buffer[..new_cursor].chars().count();

        self.handle_update_buffer(new_buffer);
        self.pending_editor_cursor = Some((new_char_cursor, new_char_cursor));
    }
}
