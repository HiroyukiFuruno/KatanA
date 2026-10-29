use crate::app::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_select_document(&mut self, p: std::path::PathBuf) {
        self.state.layout.inline_rename_group = None;
        self.handle_select_document(p, true);
        if self.state.layout.show_search_modal {
            self.state.layout.show_search_modal = false;
        }
    }
    pub(super) fn handle_action_select_and_jump(&mut self, path: std::path::PathBuf, line: usize) {
        self.state.layout.inline_rename_group = None;
        self.handle_select_document(path, true);
        if self.state.layout.show_search_modal {
            self.state.layout.show_search_modal = false;
        }
        /* WHY: FB1 — If the active document is currently in PreviewOnly mode the editor pane
         * is not rendered, so scroll_to_line would never be consumed. Force CodeOnly so the
         * editor becomes visible and the jump actually scrolls to the target line. */
        use crate::state::document::ViewMode;
        if self.state.active_view_mode() == ViewMode::PreviewOnly
            && crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                &self.state,
                ViewMode::CodeOnly,
            )
        {
            self.state.set_active_view_mode(ViewMode::CodeOnly);
        }
        self.state.scroll.scroll_to_line = Some(line);
    }

    pub(super) fn handle_action_open_multiple(&mut self, paths: Vec<std::path::PathBuf>) {
        let mut iter = paths.into_iter();
        if let Some(first_path) = iter.next() {
            self.handle_select_document(first_path, true);
        }
        for path in iter {
            self.pending_document_loads.push_back(path);
        }
    }

    pub(super) fn handle_action_close_document(&mut self, idx: usize) {
        let should_confirm = self
            .state
            .config
            .settings
            .settings()
            .behavior
            .confirm_close_dirty_tab
            && idx < self.state.document.open_documents.len()
            && self.state.document.open_documents[idx].is_dirty;
        if idx < self.state.document.open_documents.len()
            && self.state.document.open_documents[idx].is_pinned
        {
            return;
        }
        if should_confirm {
            self.state.layout.pending_close_confirm = Some(idx);
        } else {
            self.force_close_document(idx);
        }
    }
}

#[cfg(test)]
mod select_and_jump_tests {
    use crate::app::action::ActionOps;
    use crate::app_state::{AppAction, AppState};
    use crate::shell::KatanaApp;
    use crate::state::document::ViewMode;
    use katana_core::{ai::AiProviderRegistry, document::Document, plugin::PluginRegistry};
    use std::sync::Arc;

    fn app_with_open_document(path: &str) -> KatanaApp {
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = KatanaApp::new(state);
        app.state
            .document
            .open_documents
            .push(Document::new_empty(path));
        app.state.document.active_doc_idx = Some(0);
        app
    }

    fn select_and_jump(path: &str) -> KatanaApp {
        let mut app = app_with_open_document(path);
        app.process_action(
            &eframe::egui::Context::default(),
            AppAction::SelectDocumentAndJump {
                path: path.into(),
                line: 12,
                byte_range: 0..0,
            },
        );
        assert_eq!(app.state.scroll.scroll_to_line, Some(12));
        app
    }

    #[test]
    fn select_and_jump_keeps_code_mode_for_markdown_and_preview_for_binary_documents() {
        assert_eq!(
            select_and_jump("notes.md").state.active_view_mode(),
            ViewMode::CodeOnly
        );
        for path in ["manual.pdf", "report.docx", "book.xlsx", "deck.pptx"] {
            assert_eq!(
                select_and_jump(path).state.active_view_mode(),
                ViewMode::PreviewOnly,
                "{path}"
            );
        }
    }
}
