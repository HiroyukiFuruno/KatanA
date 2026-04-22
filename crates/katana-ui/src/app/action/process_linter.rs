use crate::app_state::{AppAction, StatusType};
use crate::shell::*;
use std::path::PathBuf;

impl KatanaApp {
    pub(super) fn handle_open_linter_doc(
        &mut self,
        ctx: &eframe::egui::Context,
        rule_id: String,
        docs_url: String,
    ) {
        let content_opt = self.linter_docs_cache.get(&rule_id).cloned();
        if let Some(content) = content_opt {
            self.open_virtual_linter_doc(ctx, &rule_id, &content);
            return;
        }

        let raw_url = docs_url
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/");
        /* WHY: GitHub raw file paths are case-sensitive. Rule IDs like "MD038" are uppercase,
         * but the actual doc filenames are lowercase (e.g. md038.md). Lowercase the filename. */
        let raw_url = if let Some(pos) = raw_url.rfind('/') {
            let (prefix, filename) = raw_url.split_at(pos + 1);
            format!("{}{}", prefix, filename.to_lowercase())
        } else {
            raw_url
        };

        let (tx, rx) = std::sync::mpsc::channel();
        self.linter_doc_rx = Some(rx);

        let request = ehttp::Request::get(&raw_url);
        let ctx_clone = ctx.clone();
        let rule_id_clone = rule_id.clone();

        self.state.layout.status_message = Some((
            format!("Fetching documentation for {}...", rule_id_clone),
            StatusType::Info,
        ));

        ehttp::fetch(request, move |result| match result {
            Ok(response) => {
                let res = if response.ok {
                    match response.text() {
                        Some(t) => Ok(t.to_string()),
                        None => Err("Failed to decode response text".to_string()),
                    }
                } else {
                    Err(format!("HTTP error: {}", response.status))
                };
                let _ = tx.send((rule_id_clone, res));
                ctx_clone.request_repaint();
            }
            Err(err) => {
                let _ = tx.send((rule_id_clone, Err(err)));
                ctx_clone.request_repaint();
            }
        });
    }

    pub(crate) fn poll_linter_docs(&mut self, ctx: &eframe::egui::Context) {
        if let Some(rx) = &self.linter_doc_rx {
            match rx.try_recv() {
                Ok((rule_id, Ok(content))) => {
                    self.linter_docs_cache
                        .insert(rule_id.clone(), content.clone());
                    self.state.layout.status_message = None;
                    self.open_virtual_linter_doc(ctx, &rule_id, &content);
                    self.linter_doc_rx = None;
                }
                Ok((rule_id, Err(err))) => {
                    self.state.layout.status_message = Some((
                        format!("Failed to fetch {} docs: {}", rule_id, err),
                        StatusType::Error,
                    ));
                    self.linter_doc_rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.linter_doc_rx = None;
                }
            }
        }
    }

    fn open_virtual_linter_doc(
        &mut self,
        _ctx: &eframe::egui::Context,
        rule_id: &str,
        content: &str,
    ) {
        let virtual_path = PathBuf::from(format!("Katana://LinterDocs/{}.md", rule_id));
        let mut doc = katana_core::document::Document::new_empty(&virtual_path);
        doc.buffer = content.to_string();
        doc.is_loaded = true;

        /* WHY: Check if already open */
        if let Some(idx) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|d| d.path == virtual_path)
        {
            self.state.document.open_documents[idx] = doc;
            self.state.document.active_doc_idx = Some(idx);
        } else {
            self.state.document.open_documents.push(doc);
            self.state.document.active_doc_idx = Some(self.state.document.open_documents.len() - 1);
        }

        /* WHY: Auto-switch view so the rendered markdown is visible.
         * CodeOnly → Split: user needs the preview to read the doc.
         * PreviewOnly → Split: user needs the editor/code panel to see the rendered content.
         * Split already shows both panels, so no change needed. */
        match self.state.active_view_mode() {
            crate::app_state::ViewMode::CodeOnly | crate::app_state::ViewMode::PreviewOnly => {
                self.state
                    .set_active_view_mode(crate::app_state::ViewMode::Split);
            }
            crate::app_state::ViewMode::Split => {}
        }

        /* WHY: Trigger a refresh so diagram controller parses it */
        self.pending_action = AppAction::RefreshDocument { is_manual: false };
    }
}
