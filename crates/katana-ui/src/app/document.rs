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
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) trait DocumentOps {
    fn handle_select_document(&mut self, path: std::path::PathBuf, activate: bool);
    fn force_close_document(&mut self, idx: usize);
    fn handle_update_buffer(&mut self, content: String);
    fn handle_replace_text(&mut self, span: std::ops::Range<usize>, replacement: String);
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

        if let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|d| d.path == path)
        {
            if activate {
                self.state.document.active_doc_idx = Some(idx);
                let doc = &mut self.state.document.open_documents[idx];
                if !doc.is_loaded {
                    let Ok(loaded_doc) = self.fs.load_document(&path) else {
                        return;
                    };
                    let pinned = doc.is_pinned;
                    *doc = loaded_doc;
                    doc.is_pinned = pinned;
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

        if activate {
            match self.fs.load_document(&path) {
                Ok(doc) => {
                    let src = doc.buffer.clone();
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
                    self.state.document.open_documents.push(doc);
                    self.state.document.active_doc_idx =
                        Some(self.state.document.open_documents.len() - 1);
                    self.state.initialize_tab_split_state(path.clone());
                    self.save_workspace_state();
                    self.pending_action = crate::app_state::AppAction::RefreshDiagnostics;
                }
                Err(e) => {
                    let error = e.to_string();
                    self.state.layout.status_message = Some((
                        crate::i18n::I18nOps::tf(
                            &crate::i18n::I18nOps::get().status.cannot_open_file,
                            &[("error", error.as_str())],
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                }
            }
        } else {
            self.state
                .document
                .open_documents
                .push(katana_core::document::Document::new_empty(path.clone()));
            self.state.initialize_tab_split_state(path);
            self.save_workspace_state();
        }
    }
    fn force_close_document(&mut self, idx: usize) {
        DocCloseOps::force_close_document(self, idx);
    }
    fn handle_update_buffer(&mut self, content: String) {
        let path = if let Some(doc) = self.state.active_document_mut() {
            doc.update_buffer(content.clone());
            doc.path.clone()
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(&content);
        }
    }
    fn handle_replace_text(&mut self, span: std::ops::Range<usize>, replacement: String) {
        let (path, content) = if let Some(doc) = self.state.active_document_mut() {
            if span.start <= span.end && span.end <= doc.buffer.len() {
                doc.buffer.replace_range(span, &replacement);
                doc.is_dirty = true;
            }
            (doc.path.clone(), doc.buffer.clone())
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
        if self.state.search.doc_search_open {
            self.refresh_doc_search_matches(&content);
        }
    }
    fn handle_save_document(&mut self) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        match self.fs.save_document(doc) {
            Ok(()) => {
                self.state.layout.status_message = Some((
                    crate::i18n::I18nOps::get().status.saved.clone(),
                    crate::app_state::StatusType::Success,
                ));
                self.save_workspace_state();
                self.pending_action = crate::app_state::AppAction::RefreshDiagnostics;
            }
            Err(e) => {
                let error = e.to_string();
                self.state.layout.status_message = Some((
                    crate::i18n::I18nOps::tf(
                        &crate::i18n::I18nOps::get().status.save_failed,
                        &[("error", error.as_str())],
                    ),
                    crate::app_state::StatusType::Error,
                ));
            }
        }
    }
}
