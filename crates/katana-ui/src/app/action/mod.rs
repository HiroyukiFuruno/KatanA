#![allow(dead_code)]
mod clipboard_file_url;
pub(crate) mod clipboard_image;
#[cfg(target_os = "macos")]
mod clipboard_image_macos;
mod closed_preview_memory;
mod demo_bundle;
mod dispatch;
mod dispatch_panels;
mod dispatch_secondary;
mod dispatch_tertiary;
mod file_open;
mod html_navigation;
mod image_ingest;
mod language;
mod process_authoring;
mod process_demo;
mod process_demo_group;
mod process_diagnostics;
#[cfg(test)]
mod process_diagnostics_tests;
mod process_document;
mod process_group_lifecycle;
mod process_groups;
mod process_helpers;
mod process_linter;
mod process_markdown_formatting;
mod process_markdown_formatting_diagnostics;
mod process_markdown_formatting_paths;
mod process_reorder;
mod process_tabs;
mod process_update;
mod refresh_content;
mod url_source;

#[cfg(test)]
mod process_markdown_formatting_tests;

use crate::app::*;
use crate::app_state::*;
use crate::shell::*;
pub(crate) use file_open::FileOpenOps;

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
        let removed_document_preview = self.tab_previews.iter().any(|preview| {
            !open_paths.contains(&preview.path)
                && katana_core::workspace::TreeEntry::path_is_document(&preview.path)
        });
        self.tab_previews.retain(|t| open_paths.contains(&t.path));
        self.state
            .url_tab
            .document_tabs
            .retain(|tab| open_paths.contains(&tab.document_path));
        self.state.url_tab.pending_url_requests.retain(|request| {
            request
                .target_document
                .as_ref()
                .is_none_or(|path| open_paths.contains(path))
        });
        self.state.url_tab.is_loading = !self.state.url_tab.pending_url_requests.is_empty();
        let transition = closed_preview_memory::ClosedPreviewTransition {
            removed_document_preview,
            open_documents_empty: open_paths.is_empty(),
            previews_empty: self.tab_previews.is_empty(),
        };
        if closed_preview_memory::should_relieve_memory(transition) {
            crate::preview_pane::DocumentWorkerLifecycle::request_closed_preview_memory_relief();
        } else if !transition.open_documents_empty || !transition.previews_empty {
            crate::preview_pane::DocumentWorkerLifecycle::cancel_closed_preview_memory_relief();
        }
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
        self.close_diff_review_if_tab_removed();
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
        let lang = crate::i18n::I18nOps::get_language();
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

#[cfg(test)]
mod closed_preview_memory_tests {
    use super::ActionOps;
    use crate::app_state::AppState;
    use crate::preview_pane::PreviewPane;
    use crate::shell::{KatanaApp, TabPreviewCache};
    use katana_core::document::Document;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn make_app() -> KatanaApp {
        let state = AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    fn add_preview(app: &mut KatanaApp, path: impl Into<PathBuf>) {
        app.tab_previews.push(TabPreviewCache {
            path: path.into(),
            pane: PreviewPane::default(),
            hash: 0,
        });
    }

    #[test]
    fn cleanup_removes_closed_document_preserves_remaining_content() {
        let mut app = make_app();
        let remaining = PathBuf::from("keep.docx");
        let closed = PathBuf::from("closed.docx");
        let mut document = Document::new(remaining.clone(), "unsaved content");
        document.is_dirty = true;
        app.state.document.open_documents.push(document);
        add_preview(&mut app, remaining.clone());
        add_preview(&mut app, closed);

        app.cleanup_closed_tab_previews();

        assert_eq!(app.tab_previews.len(), 1);
        assert_eq!(app.tab_previews[0].path, remaining);
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "unsaved content"
        );
        assert!(app.state.document.open_documents[0].is_dirty);
    }

    #[test]
    fn cleanup_keeps_pinned_and_second_tab_and_does_not_repeat() {
        let mut app = make_app();
        let pinned = PathBuf::from("pinned.docx");
        let second = PathBuf::from("second.docx");
        let closed = PathBuf::from("closed.docx");
        let mut pinned_doc = Document::new(pinned.clone(), "pinned content");
        pinned_doc.is_pinned = true;
        app.state.document.open_documents.push(pinned_doc);
        app.state
            .document
            .open_documents
            .push(Document::new(second.clone(), "second content"));
        add_preview(&mut app, pinned.clone());
        add_preview(&mut app, second.clone());
        add_preview(&mut app, closed);

        app.cleanup_closed_tab_previews();
        let paths = app
            .tab_previews
            .iter()
            .map(|preview| preview.path.clone())
            .collect::<Vec<_>>();
        app.cleanup_closed_tab_previews();

        assert_eq!(app.tab_previews.len(), 2);
        assert_eq!(
            paths,
            app.tab_previews
                .iter()
                .map(|p| p.path.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "pinned content"
        );
        assert_eq!(
            app.state.document.open_documents[1].buffer,
            "second content"
        );
    }

    #[test]
    fn cleanup_last_document_removes_preview_once() {
        let mut app = make_app();
        add_preview(&mut app, "closed.docx");

        app.cleanup_closed_tab_previews();
        app.cleanup_closed_tab_previews();

        assert!(app.state.document.open_documents.is_empty());
        assert!(app.tab_previews.is_empty());
    }

    #[test]
    fn cleanup_html_only_removes_preview_without_document_transition() {
        let mut app = make_app();
        add_preview(&mut app, "index.html");

        app.cleanup_closed_tab_previews();

        assert!(app.state.document.open_documents.is_empty());
        assert!(app.tab_previews.is_empty());
        assert!(!katana_core::workspace::TreeEntry::path_is_document(
            std::path::Path::new("index.html")
        ));
    }
}
