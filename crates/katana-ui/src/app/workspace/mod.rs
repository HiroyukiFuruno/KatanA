#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;
use katana_platform::CacheFacade;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) trait WorkspaceOps {
    fn handle_open_workspace(&mut self, path: std::path::PathBuf);
    fn finish_open_workspace(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    );
    fn handle_refresh_workspace(&mut self);
    fn poll_workspace_load(&mut self, ctx: &egui::Context);
    fn handle_remove_workspace(&mut self, path: String);
    fn save_workspace_state(&mut self);
}

impl WorkspaceOps for KatanaApp {
    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        if self.state.workspace.data.is_some() {
            self.save_workspace_state();
        }

        self.state.workspace.is_loading = true;
        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().status.opened_workspace,
                &[("name", "...")],
            ),
            crate::app_state::StatusType::Info,
        ));

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);
        let path_clone = path.clone();

        if let Some(token) = &self.state.workspace.cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.workspace.cancel_token = Some(new_token.clone());

        let settings = self.state.config.settings.settings().workspace.clone();
        let in_memory_dirs = self.state.workspace.in_memory_dirs.clone();

        std::thread::spawn(move || {
            let fs = katana_platform::FilesystemService::new();
            let result = fs.open_workspace(
                &path_clone,
                &settings.ignored_directories,
                settings.max_depth,
                &settings.visible_extensions,
                &settings.extensionless_excludes,
                new_token,
                &in_memory_dirs,
            );
            let _ = tx.send((WorkspaceLoadType::Open, path_clone, result));
        });
    }
    fn finish_open_workspace(
        &mut self,
        _path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    ) {
        let name = ws.name().unwrap_or("unknown").to_string();
        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().status.opened_workspace,
                &[("name", name.as_str())],
            ),
            crate::app_state::StatusType::Success,
        ));
        self.state.workspace.data = Some(ws);
        self.state.document.open_documents.clear();
        self.state.document.active_doc_idx = None;
        self.state.document.tab_groups.clear();
        self.state.document.tab_view_modes.clear();
        self.state.document.tab_split_states.clear();
        self.state.document.recently_closed_tabs.clear();
        self.state.search.filter_cache = None;
        let path_str = self
            .state
            .workspace
            .data
            .as_ref()
            .unwrap()
            .root
            .display()
            .to_string();

        let mut to_open: Vec<(String, bool)> = Vec::new();
        let mut active_idx = None;

        let workspace_root = self.state.workspace.data.as_ref().unwrap().root.clone();
        let cache_key = katana_platform::cache::PersistentKey::WorkspaceTabs {
            workspace_path: workspace_root.clone(),
        }
        .to_raw_key()
        .unwrap_or_default();

        let mut workspace_path_str = workspace_root.to_string_lossy().to_string();
        if workspace_path_str.ends_with('/') || workspace_path_str.ends_with('\\') {
            workspace_path_str.pop();
        }
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;
        let workspace_hash = {
            let mut hash: u64 = FNV_OFFSET_BASIS;
            for b in workspace_path_str.bytes() {
                hash ^= b as u64;
                hash = hash.wrapping_mul(FNV_PRIME);
            }
            hash
        };
        let state_key = format!("{:x}", workspace_hash);

        let restore_session = self
            .state
            .config
            .settings
            .settings()
            .workspace
            .restore_session;
        let mut state_json_opt = None;

        if restore_session {
            state_json_opt = self.state.config.settings.load_workspace_state(&state_key);
            if state_json_opt.is_none()
                && let Some(cached_json) = self.state.config.cache.get_persistent(&cache_key)
            {
                let _ = self
                    .state
                    .config
                    .settings
                    .save_workspace_state(&state_key, &cached_json);
                state_json_opt = Some(cached_json);
            }
        }

        let settings = self.state.config.settings.settings_mut();

        if restore_session {
            if let Some(cache_json) = state_json_opt {
                if let Ok(v2) = serde_json::from_str::<WorkspaceTabSessionV2>(&cache_json) {
                    to_open = v2.tabs.into_iter().map(|t| (t.path, t.pinned)).collect();
                    if let Some(active_path) = v2.active_path {
                        active_idx = to_open.iter().position(|(p, _)| p == &active_path);
                    }
                    self.state.workspace.expanded_directories = v2
                        .expanded_directories
                        .into_iter()
                        .map(std::path::PathBuf::from)
                        .collect();

                    self.state.document.tab_groups = v2.groups;
                } else {
                    #[derive(serde::Deserialize)]
                    struct LegacyTabState {
                        tabs: Vec<String>,
                        active_idx: Option<usize>,
                        #[serde(default)]
                        expanded_directories: std::collections::HashSet<String>,
                    }
                    match serde_json::from_str::<LegacyTabState>(&cache_json) {
                        Ok(state) => {
                            to_open = state.tabs.into_iter().map(|t| (t, false)).collect();
                            active_idx = state.active_idx;
                            self.state.workspace.expanded_directories = state
                                .expanded_directories
                                .into_iter()
                                .map(std::path::PathBuf::from)
                                .collect();
                        }
                        Err(e) => {
                            tracing::warn!("Failed to deserialize tab state: {}", e);
                        }
                    }
                }
            } else {
                let is_same_as_last =
                    settings.workspace.last_workspace.as_deref() == Some(path_str.as_str());
                if is_same_as_last {
                    to_open = settings
                        .workspace
                        .open_tabs
                        .clone()
                        .into_iter()
                        .map(|t| (t, false))
                        .collect();
                    active_idx = settings.workspace.active_tab_idx;
                }
            }
        }

        settings.workspace.last_workspace = Some(path_str.clone());
        settings.workspace.paths.retain(|p| p != &path_str);
        settings.workspace.paths.push(path_str);

        to_open.retain(|(p, _)| std::path::Path::new(p).exists());

        let existing_paths: std::collections::HashSet<String> =
            to_open.iter().map(|(p, _)| p.clone()).collect();
        for g in &mut self.state.document.tab_groups {
            g.members.retain(|m| existing_paths.contains(m));
        }
        self.state
            .document
            .tab_groups
            .retain(|g| !g.members.is_empty());

        if !to_open.is_empty() {
            let active_idx_val = active_idx.unwrap_or(0).min(to_open.len().saturating_sub(1));

            for (i, (p, pinned)) in to_open.iter().enumerate() {
                let path = std::path::PathBuf::from(p);
                if i == active_idx_val {
                    match self.fs.load_document(path) {
                        Ok(doc) => {
                            let mut doc = doc;
                            doc.is_pinned = *pinned;
                            self.state.document.open_documents.push(doc);
                            self.state
                                .initialize_tab_split_state(std::path::PathBuf::from(p));
                        }
                        Err(e) => {
                            tracing::error!("Failed to load document: {}", e);
                        }
                    }
                } else {
                    let mut doc = katana_core::document::Document::new_empty(path);
                    doc.is_pinned = *pinned;
                    self.state.document.open_documents.push(doc);
                    self.state
                        .initialize_tab_split_state(std::path::PathBuf::from(p));
                }
            }
            if !self.state.document.open_documents.is_empty() {
                let idx = active_idx
                    .unwrap_or(0)
                    .min(self.state.document.open_documents.len() - 1);
                self.state.document.active_doc_idx = Some(idx);
                let active_doc = &self.state.document.open_documents[idx];
                let src = active_doc.buffer.clone();
                let doc_path = active_doc.path.clone();
                let concurrency = self
                    .state
                    .config
                    .settings
                    .settings()
                    .performance
                    .diagram_concurrency;
                self.full_refresh_preview(&doc_path, &src, false, concurrency);
            }
        }

        if !self.state.config.try_save_settings() {
            tracing::warn!("Failed to save settings");
        }
    }
    fn handle_refresh_workspace(&mut self) {
        let Some(workspace) = &self.state.workspace.data else {
            return;
        };
        let root = workspace.root.clone();

        self.state.workspace.is_loading = true;

        let (tx, rx) = std::sync::mpsc::channel();
        self.workspace_rx = Some(rx);

        if let Some(token) = &self.state.workspace.cancel_token {
            token.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let new_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.state.workspace.cancel_token = Some(new_token.clone());

        let settings = self.state.config.settings.settings().workspace.clone();
        let in_memory_dirs = self.state.workspace.in_memory_dirs.clone();

        std::thread::spawn(move || {
            let fs = katana_platform::FilesystemService::new();
            let result = fs.open_workspace(
                &root,
                &settings.ignored_directories,
                settings.max_depth,
                &settings.visible_extensions,
                &settings.extensionless_excludes,
                new_token,
                &in_memory_dirs,
            );
            let _ = tx.send((WorkspaceLoadType::Refresh, root, result));
        });
    }
    fn poll_workspace_load(&mut self, ctx: &egui::Context) {
        const WORKSPACE_LOAD_POLL_INTERVAL_MS: u64 = 50;
        let done = if let Some(rx) = &self.workspace_rx {
            match rx.try_recv() {
                Ok((WorkspaceLoadType::Open, path, Ok(ws))) => {
                    self.state.workspace.is_loading = false;
                    self.finish_open_workspace(path, ws);
                    true
                }
                Ok((WorkspaceLoadType::Refresh, _path, Ok(ws))) => {
                    self.state.workspace.is_loading = false;
                    self.state.workspace.data = Some(ws);
                    self.state.search.filter_cache = None;
                    true
                }
                Ok((_load_type, _path, Err(e))) => {
                    self.state.workspace.is_loading = false;
                    let error = e.to_string();
                    self.state.layout.status_message = Some((
                        crate::i18n::I18nOps::tf(
                            &crate::i18n::I18nOps::get().status.cannot_open_workspace,
                            &[("error", error.as_str())],
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        WORKSPACE_LOAD_POLL_INTERVAL_MS,
                    ));
                    false
                }
                Err(_) => {
                    self.state.workspace.is_loading = false;
                    true
                }
            }
        } else {
            false
        };
        if done {
            self.workspace_rx = None;
        }

        if self.needs_changelog_display
            && !self.state.workspace.is_loading
            && self.workspace_rx.is_none()
            && matches!(self.pending_action, AppAction::None)
        {
            self.needs_changelog_display = false;
            self.pending_action = AppAction::ShowReleaseNotes;
        }
    }
    fn handle_remove_workspace(&mut self, path: String) {
        let settings = self.state.config.settings.settings_mut();
        settings.workspace.paths.retain(|p| p != &path);

        if settings.workspace.last_workspace.as_deref() == Some(path.as_str()) {
            settings.workspace.last_workspace = None;
        }

        if !self.state.config.try_save_settings() {
            tracing::warn!("Failed to save settings after removing workspace");
        }
    }
    fn save_workspace_state(&mut self) {
        let open_tabs: Vec<String> = self
            .state
            .document
            .open_documents
            .iter()
            .map(|d| d.path.display().to_string())
            .filter(|p| !p.starts_with("Katana://"))
            .collect();
        let idx = self.state.document.active_doc_idx;
        let expanded: std::collections::HashSet<String> = self
            .state
            .workspace
            .expanded_directories
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        let settings = self.state.config.settings.settings_mut();
        settings.workspace.open_tabs = open_tabs.clone();
        settings.workspace.active_tab_idx = idx;
        if !self.state.config.try_save_settings() {
            tracing::warn!("Failed to save workspace tab state");
        }

        if let Some(ws) = &self.state.workspace.data {
            let mut workspace_path_str = ws.root.to_string_lossy().to_string();
            if workspace_path_str.ends_with('/') || workspace_path_str.ends_with('\\') {
                workspace_path_str.pop();
            }
            let state_key = {
                const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
                const FNV_PRIME: u64 = 0x100000001b3;
                let mut hash: u64 = FNV_OFFSET_BASIS;
                for b in workspace_path_str.bytes() {
                    hash ^= b as u64;
                    hash = hash.wrapping_mul(FNV_PRIME);
                }
                format!("{:x}", hash)
            };

            let tabs_v2: Vec<WorkspaceTabEntry> = self
                .state
                .document
                .open_documents
                .iter()
                .filter(|d| !d.path.display().to_string().starts_with("Katana://"))
                .map(|d| WorkspaceTabEntry {
                    path: d.path.display().to_string(),
                    pinned: d.is_pinned,
                })
                .collect();

            let active_path = self
                .state
                .document
                .active_document()
                .map(|d| d.path.display().to_string());

            let state = WorkspaceTabSessionV2 {
                version: 2,
                tabs: tabs_v2,
                active_path,
                expanded_directories: expanded,
                groups: self.state.document.tab_groups.clone(),
            };
            match serde_json::to_string(&state) {
                Ok(json) => {
                    if let Err(e) = self
                        .state
                        .config
                        .settings
                        .save_workspace_state(&state_key, &json)
                    {
                        tracing::warn!("Failed to save workspace state: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to serialize tab state: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
