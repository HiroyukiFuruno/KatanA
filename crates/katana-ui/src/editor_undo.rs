use std::path::Path;

use eframe::egui;

pub(crate) struct EditorUndoIdentity;

impl EditorUndoIdentity {
    const TEXT_EDIT_ID: &'static str = "editor_text_edit";

    pub(crate) fn text_edit_id(workspace_root: Option<&Path>, document_path: &Path) -> egui::Id {
        match workspace_root {
            Some(root) => egui::Id::new((Self::TEXT_EDIT_ID, root, document_path)),
            None => egui::Id::new((Self::TEXT_EDIT_ID, document_path)),
        }
    }
}

pub(crate) struct EditorUndoOps;

impl EditorUndoOps {
    pub(crate) fn record_external_change(
        ctx: &egui::Context,
        workspace_root: Option<&Path>,
        document_path: &Path,
        before: &str,
        after: &str,
    ) {
        if before == after {
            return;
        }

        let id = EditorUndoIdentity::text_edit_id(workspace_root, document_path);
        let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
        let before_cursor = Self::cursor_for_text(&state, before);
        let after_cursor = Self::cursor_for_text(&state, after);
        let mut undoer = state.undoer();
        undoer.add_undo(&(before_cursor, before.to_string()));
        state.cursor.set_char_range(Some(after_cursor));
        state.set_undoer(undoer);
        egui::TextEdit::store_state(ctx, id, state);
    }

    fn cursor_for_text(
        state: &egui::widgets::text_edit::TextEditState,
        text: &str,
    ) -> egui::text::CCursorRange {
        let max = text.chars().count();
        let current = state
            .cursor
            .char_range()
            .unwrap_or_else(|| egui::text::CCursorRange::one(egui::text::CCursor::new(max)));
        Self::clamp_cursor(current, max)
    }

    fn clamp_cursor(mut range: egui::text::CCursorRange, max: usize) -> egui::text::CCursorRange {
        range.primary.index = range.primary.index.min(max);
        range.secondary.index = range.secondary.index.min(max);
        range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_edit_id_is_scoped_by_workspace_and_file() {
        let file = Path::new("/workspace/a/doc.md");
        assert_ne!(
            EditorUndoIdentity::text_edit_id(Some(Path::new("/workspace/a")), file),
            EditorUndoIdentity::text_edit_id(Some(Path::new("/workspace/b")), file)
        );
        assert_ne!(
            EditorUndoIdentity::text_edit_id(Some(Path::new("/workspace/a")), file),
            EditorUndoIdentity::text_edit_id(
                Some(Path::new("/workspace/a")),
                Path::new("/workspace/a/other.md"),
            )
        );
    }

    #[test]
    fn external_change_records_previous_content_as_undo_point() {
        let ctx = egui::Context::default();
        let workspace = Path::new("/workspace");
        let path = Path::new("/workspace/doc.md");

        EditorUndoOps::record_external_change(&ctx, Some(workspace), path, "before", "after");

        let id = EditorUndoIdentity::text_edit_id(Some(workspace), path);
        let state = egui::TextEdit::load_state(&ctx, id).expect("state must be stored");
        let mut undoer = state.undoer();
        let current = (
            egui::text::CCursorRange::one(egui::text::CCursor::new("after".chars().count())),
            "after".to_string(),
        );
        let restored = undoer.undo(&current).expect("undo point must exist");
        assert_eq!(restored.1, "before");
    }
}
