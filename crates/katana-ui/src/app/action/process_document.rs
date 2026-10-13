use crate::app::*;
use crate::shell::*;
use std::path::PathBuf;

impl KatanaApp {
    pub(super) fn handle_action_create_fs_node(
        &mut self,
        parent_dir: PathBuf,
        is_dir: bool,
        target_path: PathBuf,
    ) {
        let res = if is_dir {
            std::fs::create_dir(&target_path)
        } else {
            std::fs::File::create(&target_path).map(|_| ())
        };
        if let Err(e) = res {
            tracing::error!("Failed to create fs node: {}", e);
        } else {
            if is_dir {
                self.state.workspace.in_memory_dirs.insert(target_path);
            }
            self.handle_refresh_explorer();
            self.state.workspace.expanded_directories.insert(parent_dir);
        }
    }

    pub(super) fn handle_action_rename_fs_node(&mut self, target_path: PathBuf, new_path: PathBuf) {
        if let Err(e) = std::fs::rename(&target_path, &new_path) {
            tracing::error!("Failed to rename file: {}", e);
        } else {
            self.handle_refresh_explorer();
            for doc in &mut self.state.document.open_documents {
                if doc.path == target_path {
                    doc.path = new_path.clone();
                    break;
                }
            }
        }
    }

    pub(super) fn handle_action_delete_fs_node(&mut self, target_path: PathBuf) {
        let res = if target_path.is_dir() {
            std::fs::remove_dir_all(&target_path)
        } else {
            std::fs::remove_file(&target_path)
        };
        if let Err(e) = res {
            tracing::error!("Failed to delete path: {}", e);
            return;
        }
        self.handle_refresh_explorer();
        self.state.diagnostics.remove_file_diagnostics(&target_path); /* WHY: FB30 */
        let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|d| d.path == target_path)
        else {
            return;
        };
        self.state.document.open_documents.remove(idx);
        if let Some(active_idx) = self.state.document.active_doc_idx {
            self.state.document.active_doc_idx = Self::compute_new_active_idx(
                active_idx,
                idx,
                self.state.document.open_documents.len(),
            );
        }
        self.state.document.cleanup_empty_groups();
    }

    pub(super) fn handle_action_request_move_fs_node(
        &mut self,
        source_path: PathBuf,
        target_dir: PathBuf,
    ) {
        if source_path == target_dir || target_dir.starts_with(&source_path) {
            return;
        }
        if self
            .state
            .config
            .settings
            .settings()
            .behavior
            .confirm_file_move
        {
            self.state.layout.move_modal = Some((source_path, target_dir));
            return;
        }
        let Some(file_name) = source_path.file_name() else {
            return;
        };
        self.handle_action_move_fs_node(source_path.clone(), target_dir.join(file_name));
    }

    pub(super) fn handle_action_move_fs_node(
        &mut self,
        source_path: PathBuf,
        target_path: PathBuf,
    ) {
        if let Err(error) = std::fs::rename(&source_path, &target_path) {
            tracing::error!("Failed to move file: {}", error);
            let error_text = error.to_string();
            let message = crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().status.move_failed,
                &[("error", error_text.as_str())],
            );
            self.state.layout.status_message = Some((message, crate::app_state::StatusType::Error));
            return;
        }
        for doc in &mut self.state.document.open_documents {
            crate::app::image_document::ImageDocumentOps::refresh_reference_path(
                doc,
                &source_path,
                &target_path,
            );
        }
        let source = source_path.display().to_string();
        let target = target_path.display().to_string();
        let message = crate::i18n::I18nOps::tf(
            &crate::i18n::I18nOps::get().status.moved_file,
            &[("source", source.as_str()), ("target", target.as_str())],
        );
        self.state.layout.status_message = Some((message, crate::app_state::StatusType::Success));
        self.handle_refresh_explorer();
    }

    fn compute_new_active_idx(
        active_idx: usize,
        removed_idx: usize,
        docs_len: usize,
    ) -> Option<usize> {
        if active_idx == removed_idx {
            if docs_len == 0 {
                None
            } else {
                Some(removed_idx.saturating_sub(1))
            }
        } else if active_idx > removed_idx {
            Some(active_idx - 1)
        } else {
            Some(active_idx)
        }
    }
    pub(super) fn handle_action_select_document(&mut self, p: std::path::PathBuf) {
        self.handle_select_document(p, true);
        if self.state.layout.show_search_modal {
            self.state.layout.show_search_modal = false;
        }
    }
    pub(super) fn handle_action_select_and_jump(&mut self, path: std::path::PathBuf, line: usize) {
        self.handle_select_document(path, true);
        if self.state.layout.show_search_modal {
            self.state.layout.show_search_modal = false;
        }
        /* WHY: FB1 — If the active document is currently in PreviewOnly mode the editor pane
         * is not rendered, so scroll_to_line would never be consumed. Force CodeOnly so the
         * editor becomes visible and the jump actually scrolls to the target line. */
        use crate::state::document::ViewMode;
        if self.state.active_view_mode() == ViewMode::PreviewOnly {
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
