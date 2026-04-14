#![allow(dead_code)]
mod demo_bundle;
mod dispatch;
mod dispatch_secondary;
mod image_ingest;
mod process_authoring;
mod process_demo;
mod process_document;
mod process_group_lifecycle;
mod process_groups;
mod process_helpers;
mod process_reorder;
mod process_tabs;
mod process_update;
mod refresh_content;

use crate::app::*;
use crate::app_state::*;
use crate::shell::*;

pub(crate) trait ActionOps {
    fn take_action(&mut self) -> AppAction;
    fn handle_toggle_task_list(&mut self, global_index: usize, new_state: char);
    fn cleanup_closed_tab_previews(&mut self);

    fn process_action(&mut self, ctx: &egui::Context, action: AppAction);
    fn handle_show_release_notes(&mut self);
    fn poll_changelog(&mut self, _ctx: &egui::Context);
    fn trigger_action(&mut self, action: AppAction);
    fn app_state_mut(&mut self) -> &mut AppState;
}

impl ActionOps for KatanaApp {
    fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }

    fn handle_toggle_task_list(&mut self, global_index: usize, new_state: char) {
        let (path, content) = if let Some(doc) = self.state.active_document_mut() {
            let spans = egui_commonmark::extract_task_list_spans(&doc.buffer);
            if let Some(span) = spans.get(global_index) {
                let replacement = format!("[{}]", new_state);
                if span.start <= span.end && span.end <= doc.buffer.len() {
                    doc.buffer.replace_range(span.clone(), &replacement);
                    doc.is_dirty = true;
                }
            } else {
                tracing::warn!(
                    "Interactive Task List out of bounds: global_index {} vs {}",
                    global_index,
                    spans.len()
                );
            }
            (doc.path.clone(), doc.buffer.clone())
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
    }

    fn cleanup_closed_tab_previews(&mut self) {
        let open_paths: std::collections::HashSet<_> = self
            .state
            .document
            .open_documents
            .iter()
            .map(|d| &d.path)
            .collect();
        self.tab_previews.retain(|t| open_paths.contains(&t.path));
    }

    fn process_action(&mut self, ctx: &egui::Context, action: AppAction) {
        self.dispatch_action(ctx, action);
        self.cleanup_closed_tab_previews();
        let mut inactive_but_focused_path = None;
        if let Some(active_idx) = self.state.document.active_doc_idx
            && let Some(doc) = self.state.document.open_documents.get(active_idx)
        {
            let has_preview = self.tab_previews.iter().any(|t| t.path == doc.path);
            if !doc.is_loaded || !has_preview {
                inactive_but_focused_path = Some(doc.path.clone());
            }
        }
        if let Some(path) = inactive_but_focused_path {
            self.handle_select_document(path, true);
        }
    }

    fn handle_show_release_notes(&mut self) {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        let previous = self.old_app_version.clone().or_else(|| {
            self.state
                .config
                .settings
                .settings()
                .updates
                .previous_app_version
                .clone()
        });
        let lang = self.state.config.settings.settings().language.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.changelog_rx = Some(rx);
        crate::changelog::ChangelogOps::fetch_changelog(&lang, current_version, previous, tx);
        tracing::info!("Triggered ShowReleaseNotes background fetch.");
        let virtual_path =
            std::path::PathBuf::from(format!("Katana://ChangeLog v{}", env!("CARGO_PKG_VERSION")));
        if !self
            .state
            .document
            .open_documents
            .iter()
            .any(|d| d.path == virtual_path)
        {
            self.state
                .document
                .open_documents
                .push(katana_core::document::Document::new_empty(
                    virtual_path.clone(),
                ));
        }
        self.handle_select_document(virtual_path, true);
    }

    fn poll_changelog(&mut self, _ctx: &egui::Context) {
        let Some(rx) = &self.changelog_rx else { return };
        let Ok(event) = rx.try_recv() else { return };
        self.changelog_rx = None;
        match event {
            crate::changelog::ChangelogEvent::Success(sections) => {
                self.changelog_sections = sections;
                let virtual_path = std::path::PathBuf::from(format!(
                    "Katana://ChangeLog v{}",
                    env!("CARGO_PKG_VERSION")
                ));
                if let Some(pos) = self
                    .state
                    .document
                    .open_documents
                    .iter()
                    .position(|d| d.path == virtual_path)
                {
                    self.state.document.active_doc_idx = Some(pos);
                } else {
                    self.state
                        .document
                        .open_documents
                        .push(katana_core::document::Document::new_empty(virtual_path));
                    self.state.document.active_doc_idx =
                        Some(self.state.document.open_documents.len() - 1);
                }
            }
            crate::changelog::ChangelogEvent::Error(err) => {
                tracing::error!("Failed to fetch changelog: {}", err);
                self.state.layout.status_message = Some((
                    format!("Failed to fetch release notes: {err}"),
                    StatusType::Error,
                ));
            }
        }
    }

    fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }

    fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}
