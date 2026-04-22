#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::app::doc_close::DocCloseOps;
use crate::app::doc_search::DocSearchRefresh;
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;

pub(crate) trait DocumentOps {
    fn handle_select_document(&mut self, path: std::path::PathBuf, activate: bool);
    fn force_close_document(&mut self, idx: usize);
    fn handle_update_buffer(&mut self, content: String);
    fn handle_save_document(&mut self);
}

impl DocumentOps for KatanaApp {
    fn handle_select_document(&mut self, path: std::path::PathBuf, activate: bool) {
        if activate {
            let mut parent = path.parent();
            while let Some(p) = parent {
                if p == std::path::Path::new("") {
                    break;
                }
                self.state
                    .workspace
                    .expanded_directories
                    .insert(p.to_path_buf());
                parent = p.parent();
            }
        }

        /* WHY: Check if already open. If so, just activate if requested. */
        if let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|d| d.path == path)
        {
            if activate {
                self.state.document.active_doc_idx = Some(idx);

                /* WHY: Load if needed. */
                let doc_is_loaded = self.state.document.open_documents[idx].is_loaded;
                if !doc_is_loaded
                    && !path.to_string_lossy().starts_with("Katana://")
                    && let Ok(mut loaded_doc) = self.fs.load_document(&path)
                {
                    let pinned = self.state.document.open_documents[idx].is_pinned;
                    loaded_doc.is_pinned = pinned;
                    self.state.document.open_documents[idx] = loaded_doc;
                }

                let src = self.state.document.open_documents[idx].buffer.clone();
                let concurrency = self
                    .state
                    .config
                    .settings
                    .settings()
                    .performance
                    .diagram_concurrency;
                self.full_refresh_preview(&path, &src, false, concurrency);
                if self.state.search.doc_search_open {
                    self.refresh_doc_search_matches(&src);
                }
                self.pending_action = crate::app_state::AppAction::RefreshDiagnostics;
            }
            return;
        }

        /* WHY: Not open yet. Load and add to open documents. */
        let doc = if activate {
            match self.fs.load_document(&path) {
                Ok(d) => d,
                Err(e) => {
                    self.state.layout.status_message = Some((
                        crate::i18n::I18nOps::tf(
                            &crate::i18n::I18nOps::get().status.cannot_open_file,
                            &[("error", e.to_string().as_str())],
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                    return;
                }
            }
        } else {
            katana_core::document::Document::new_empty(&path)
        };

        let src = doc.buffer.clone();
        let (concurrency, search_open) = {
            let settings = self.state.config.settings.settings();
            (
                settings.performance.diagram_concurrency,
                self.state.search.doc_search_open,
            )
        };
        self.full_refresh_preview(&path, &src, false, concurrency);
        self.state.document.open_documents.push(doc);
        if activate {
            self.state.document.active_doc_idx = Some(self.state.document.open_documents.len() - 1);
            if search_open {
                /* WHY: Refresh search matches only when activating a document.
                Background loading must not disrupt the current UI state. */
                self.refresh_doc_search_matches(&src);
            }
        }
        self.state.initialize_tab_split_state(path.clone());
        self.save_workspace_state();
        self.pending_action = crate::app_state::AppAction::RefreshDiagnostics;
    }

    fn force_close_document(&mut self, idx: usize) {
        DocCloseOps::force_close_document(self, idx);
    }

    fn handle_update_buffer(&mut self, content: String) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let doc = &mut self.state.document.open_documents[idx];
        doc.buffer = content.clone();
        doc.is_dirty = true;

        let path = doc.path.clone();
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        self.full_refresh_preview(&path, &content, true, concurrency);

        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(&content);
        }
        self.state.diagnostics.last_buffer_update = Some(std::time::Instant::now());
    }

    fn handle_save_document(&mut self) {
        let Some(idx) = self.state.document.active_doc_idx else {
            return;
        };
        let doc = &mut self.state.document.open_documents[idx];
        if !doc.is_dirty {
            return;
        }

        if let Err(e) = self.fs.save_document(doc) {
            self.state.layout.status_message = Some((
                crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().status.save_failed,
                    &[("error", &e.to_string())],
                ),
                crate::app_state::StatusType::Error,
            ));
            return;
        }

        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::get().status.saved.clone(),
            crate::app_state::StatusType::Success,
        ));
    }
}
