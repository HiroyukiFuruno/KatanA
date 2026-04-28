use crate::app_state::{AppAction, StatusType};
use crate::linter_docs::LinterDocIdentity;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_open_linter_doc(
        &mut self,
        ctx: &eframe::egui::Context,
        rule_id: String,
        docs_url: String,
    ) {
        let identity = LinterDocIdentity::for_current_language(&rule_id);
        let cache_key = identity.cache_key();
        let content_opt = self.linter_docs_cache.get(&cache_key).cloned();
        if let Some(content) = content_opt {
            self.open_virtual_linter_doc(ctx, &identity, &content);
            return;
        }

        let raw_url = identity.raw_url(&docs_url);
        let (tx, rx) = std::sync::mpsc::channel();
        self.linter_doc_rx = Some(rx);

        let request = ehttp::Request::get(&raw_url);
        let ctx_clone = ctx.clone();
        let cache_key_clone = cache_key.clone();
        let status_label = identity.status_label();

        self.state.layout.status_message = Some((
            format!("Fetching documentation for {status_label}..."),
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
                let _ = tx.send((cache_key_clone, res));
                ctx_clone.request_repaint();
            }
            Err(err) => {
                let _ = tx.send((cache_key_clone, Err(err)));
                ctx_clone.request_repaint();
            }
        });
    }

    pub(crate) fn poll_linter_docs(&mut self, ctx: &eframe::egui::Context) {
        if let Some(rx) = &self.linter_doc_rx {
            match rx.try_recv() {
                Ok((cache_key, Ok(content))) => {
                    let identity = LinterDocIdentity::from_cache_key(&cache_key);
                    self.linter_docs_cache
                        .insert(cache_key.clone(), content.clone());
                    self.state.layout.status_message = None;
                    self.open_virtual_linter_doc(ctx, &identity, &content);
                    self.linter_doc_rx = None;
                }
                Ok((cache_key, Err(err))) => {
                    let identity = LinterDocIdentity::from_cache_key(&cache_key);
                    self.state.layout.status_message = Some((
                        format!("Failed to fetch {} docs: {}", identity.status_label(), err),
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
        identity: &LinterDocIdentity,
        content: &str,
    ) {
        let virtual_path = identity.virtual_path();
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

        /* WHY: FB8 - Always use PreviewOnly for LinterDocs, never allow split or code view */
        self.state
            .set_active_view_mode(crate::app_state::ViewMode::PreviewOnly);

        /* WHY: Trigger a refresh so diagram controller parses it */
        self.pending_action = AppAction::RefreshDocument { is_manual: false };
    }
}
