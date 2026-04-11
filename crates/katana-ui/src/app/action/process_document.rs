use crate::app::*;
use crate::app_state::StatusType;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_create_fs_node(
        &mut self,
        parent_dir: std::path::PathBuf,
        is_dir: bool,
        target_path: std::path::PathBuf,
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

    pub(super) fn handle_action_rename_fs_node(
        &mut self,
        target_path: std::path::PathBuf,
        new_path: std::path::PathBuf,
    ) {
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

    pub(super) fn handle_action_delete_fs_node(&mut self, target_path: std::path::PathBuf) {
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
        self.state.scroll.scroll_to_line = Some(line);
        self.state.scroll.preview_search_scroll_pending = true;
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

    pub(super) fn handle_action_refresh_diagrams(&mut self, ctx: &egui::Context) {
        ctx.forget_all_images();
        crate::icon::IconRegistry::install(ctx);
        for tab in &mut self.tab_previews {
            tab.hash = 0;
            for viewer in tab.pane.viewer_states.iter_mut() {
                viewer.texture = None;
            }
            tab.pane.fullscreen_viewer_state.texture = None;
        }
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        let path = doc.path.clone();
        let src = doc.buffer.clone();
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        self.full_refresh_preview(&path, &src, true, concurrency);
    }

    pub(super) fn handle_action_refresh_document(&mut self, ctx: &egui::Context, is_manual: bool) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let path = self.state.document.open_documents[idx].path.clone();
        if path.to_string_lossy().starts_with("Katana://") {
            if is_manual {
                /* WHY: For virtual documents (like Demo), there is no filesystem file to read.
                However, if the user explicitly clicked Refresh, they likely installed
                missing dependencies (like mmdc or Java) and want to retry rendering. */
                self.handle_action_refresh_diagrams(ctx);
            }
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(new_content) => {
                self.apply_refreshed_content(ctx, idx, &path, new_content, is_manual)
            }
            Err(e) => {
                let msg = crate::i18n::I18nOps::get()
                    .status
                    .refresh_failed
                    .replace("{error}", &e.to_string());
                self.state.layout.status_message = Some((msg, StatusType::Error));
            }
        }
    }
}
