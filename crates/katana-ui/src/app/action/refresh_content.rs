use crate::app::image_document::ImageDocumentOps;
use crate::app::*;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    /// Apply freshly read file content, handling hash checks and dirty-doc logic.
    pub(super) fn apply_refreshed_content(
        &mut self,
        ctx: &egui::Context,
        idx: usize,
        path: &std::path::Path,
        new_content: String,
        is_manual: bool,
    ) {
        let new_hash = katana_core::document::DocumentOps::compute_hash(&new_content);
        let mut did_update_buffer = false;
        let doc = &mut self.state.document.open_documents[idx];
        if doc.last_imported_disk_hash == Some(new_hash) {
            if is_manual {
                self.state.layout.status_message = Some((
                    crate::i18n::I18nOps::get()
                        .status
                        .refresh_no_changes
                        .clone(),
                    StatusType::Success,
                ));
            }
        } else if doc.is_dirty {
            if doc.pending_dirty_warning_hash != Some(new_hash) {
                doc.pending_dirty_warning_hash = Some(new_hash);
                self.state.layout.status_message = Some((
                    crate::i18n::I18nOps::get()
                        .status
                        .refresh_skipped_dirty
                        .clone(),
                    StatusType::Warning,
                ));
            }
        } else {
            doc.buffer = new_content.clone();
            doc.last_imported_disk_hash = Some(new_hash);
            doc.pending_dirty_warning_hash = None;
            self.state.layout.status_message = Some((
                crate::i18n::I18nOps::get().status.refresh_success.clone(),
                StatusType::Success,
            ));
            did_update_buffer = true;
        }
        if is_manual {
            self.reset_preview_caches(ctx);
        }
        if did_update_buffer || is_manual {
            let concurrency = self
                .state
                .config
                .settings
                .settings()
                .performance
                .diagram_concurrency;
            let buffer_clone = self.state.document.open_documents[idx].buffer.clone();
            self.full_refresh_preview(path, &buffer_clone, true, concurrency);
        }
    }

    fn reset_preview_caches(&mut self, ctx: &egui::Context) {
        use katana_platform::cache::CacheFacade;
        katana_platform::cache::DefaultCacheService::default().clear_diagram_cache();
        ctx.forget_all_images();
        crate::icon::IconRegistry::install(ctx);
        for tab in &mut self.tab_previews {
            tab.hash = 0;
            for viewer in tab.pane.viewer_states.iter_mut() {
                viewer.texture = None;
            }
            tab.pane.fullscreen_viewer_state.texture = None;
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
                /* WHY: For virtual documents, manual refresh should retry rendering after missing dependencies are installed. */
                self.handle_action_refresh_diagrams(ctx);
            }
            return;
        }
        if let Some((src, concurrency)) = ImageDocumentOps::refresh_payload(&self.state, idx, &path)
        {
            self.full_refresh_preview(&path, &src, true, concurrency);
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
