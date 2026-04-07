use crate::app::*;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use std::path::{Path, PathBuf};

impl KatanaApp {
    /// Handler for AppAction::OpenHelpDemo
    pub(super) fn handle_action_open_help_demo(&mut self) {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut feature_dir = current_dir.join("assets").join("feature");

        if !feature_dir.is_dir() {
            /* WHY: Fallback for tests running in `crates/katana-ui` */
            feature_dir = current_dir.join("../../assets").join("feature");
        }

        if !feature_dir.is_dir() {
            let msg = format!("Demo bundle not found at {}", feature_dir.display());
            tracing::error!("{msg}");
            self.state.layout.status_message = Some((msg, StatusType::Error));
            return;
        }

        let lang = self.state.config.settings.settings().language.clone();
        let demo_files = Self::resolve_demo_bundle(&feature_dir, &lang);

        if demo_files.is_empty() {
            let msg = "No demo files found in the bundle.".to_string();
            tracing::warn!("{msg}");
            self.state.layout.status_message = Some((msg, StatusType::Warning));
            return;
        }

        /* WHY: Open files in background and create/refresh "demo" tab group */
        self.open_demo_group(demo_files);
    }

    fn resolve_markdown_lang(path: &Path, feature_dir: &Path, name: &str, lang: &str) -> PathBuf {
        if lang == "ja" {
            let stem = name.strip_suffix(".md").unwrap_or(name);
            let ja_variant = feature_dir.join(format!("{stem}.ja.md"));
            if ja_variant.exists() {
                return ja_variant;
            }
        }
        path.to_path_buf()
    }

    /// Resolve the demo bundle from `assets/feature`.
    ///
    /// Resolution rules:
    /// - Markdown files: prefer `<name>.ja.md` when `lang == "ja"`, fall back to `<name>.md`.
    /// - Non-Markdown text files: opened as-is (reference mode).
    fn resolve_demo_bundle(feature_dir: &Path, lang: &str) -> Vec<(PathBuf, bool)> {
        let mut markdown_files: Vec<PathBuf> = Vec::new();

        let entries = std::fs::read_dir(feature_dir)
            .into_iter()
            .flatten()
            .flatten();

        for entry in entries {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if ext == "md" {
                /* WHY: Skip `.ja.md` files; we handle them via locale resolution. */
                if name.ends_with(".ja.md") {
                    continue;
                }
                let resolved = Self::resolve_markdown_lang(&path, feature_dir, name, lang);
                markdown_files.push(resolved);
            }
        }

        markdown_files.sort();

        /* WHY: Put welcome first if it exists */
        let mut sorted_md = Vec::new();
        if let Some(pos) = markdown_files.iter().position(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with("welcome")
        }) {
            sorted_md.push(markdown_files.remove(pos));
        }
        sorted_md.extend(markdown_files);

        let mut results = Vec::new();
        for p in sorted_md {
            /* WHY: All demo files are loaded in reference (read-only) mode */
            results.push((p, true));
        }

        results
    }

    fn open_demo_group(&mut self, demo_files: Vec<(PathBuf, bool)>) {
        let demo_group_name = "Demo";
        let demo_group_id = "demo".to_string();

        /* WHY: Ensure the DEMO group exists */
        if !self
            .state
            .document
            .tab_groups
            .iter()
            .any(|g| g.id == demo_group_id)
        {
            self.state
                .document
                .tab_groups
                .push(crate::state::document::TabGroup {
                    id: demo_group_id.clone(),
                    name: demo_group_name.to_string(),
                    color_hex: "#808080".to_string(), // System/Demo Grey
                    collapsed: false,
                    members: Vec::new(),
                });
        }

        /* WHY: Open the files and add to the group */
        let mut first_opened_idx = None;
        for (path, is_ref) in demo_files {
            /* WHY: Check if already open */
            let mut found_idx = None;
            for (i, doc) in self.state.document.open_documents.iter_mut().enumerate() {
                if doc.path == path {
                    /* WHY: Update reference state if it was opened outside */
                    doc.is_reference = is_ref;
                    found_idx = Some(i);
                    break;
                }
            }

            let idx = if let Some(i) = found_idx {
                i
            } else {
                /* WHY: Not open, so open it */
                let mut doc = katana_core::document::Document::new_empty(path.clone());
                doc.is_reference = is_ref;

                /* WHY: For Task 2: To make them reference docs. */
                self.pending_document_loads.push_back(path.clone());

                self.state.document.open_documents.push(doc);
                self.state.document.open_documents.len() - 1
            };

            if first_opened_idx.is_none() {
                first_opened_idx = Some(idx);
            }

            /* WHY: Assign to group */
            let path_str = path.to_string_lossy().to_string();
            if let Some(group) = self
                .state
                .document
                .tab_groups
                .iter_mut()
                .find(|g| g.id == demo_group_id)
                && !group.members.contains(&path_str)
            {
                group.members.push(path_str);
            }
        }

        /* WHY: Focus the first file (e.g. welcome.md) */
        if let Some(idx) = first_opened_idx {
            self.state.document.active_doc_idx = Some(idx);
            let path = self.state.document.open_documents[idx].path.clone();
            self.handle_select_document(path, true);
        }
    }
}
