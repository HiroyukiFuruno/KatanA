use crate::app::preview::PreviewOps;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use std::path::PathBuf;

impl KatanaApp {
    /// Handler for AppAction::OpenHelpDemo
    ///
    /// Opens all demo assets from the compile-time embedded bundle.
    /// Files use `Katana://Demo/` virtual paths so that auto-refresh
    /// and save operations are safely bypassed.
    pub(super) fn handle_action_open_help_demo(&mut self) {
        let lang = self.state.config.settings.settings().language.clone();
        let demo_assets = super::demo_bundle::resolve_demo_bundle(&lang);

        if demo_assets.is_empty() {
            let msg = "No demo files found in the bundle.".to_string();
            tracing::warn!("{msg}");
            self.state.layout.status_message = Some((msg, StatusType::Warning));
            return;
        }

        self.open_demo_group(demo_assets);
    }

    fn open_demo_group(&mut self, demo_assets: Vec<super::demo_bundle::DemoAsset>) {
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

        /* WHY: Open the embedded assets and add to the group */
        let mut first_opened_idx = None;
        for asset in demo_assets {
            let path = PathBuf::from(asset.virtual_path);

            /* WHY: Check if already open */
            let mut found_idx = None;
            for (i, doc) in self.state.document.open_documents.iter_mut().enumerate() {
                if doc.path == path {
                    /* WHY: Update reference state if it was opened outside */
                    doc.is_reference = asset.is_reference;
                    found_idx = Some(i);
                    break;
                }
            }

            let idx = if let Some(i) = found_idx {
                i
            } else {
                /* WHY: Create document directly from embedded content.
                No filesystem read needed — content is already in the binary. */
                let mut doc = katana_core::document::Document::new(path.clone(), asset.content);
                doc.is_reference = asset.is_reference;

                self.state.document.open_documents.push(doc);
                self.state.initialize_tab_split_state(path.clone());
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

        /* WHY: Focus the first file (welcome.md) and trigger preview rendering */
        if let Some(idx) = first_opened_idx {
            self.state.document.active_doc_idx = Some(idx);
            let path = self.state.document.open_documents[idx].path.clone();
            let src = self.state.document.open_documents[idx].buffer.clone();
            let concurrency = self
                .state
                .config
                .settings
                .settings()
                .performance
                .diagram_concurrency;
            self.full_refresh_preview(&path, &src, false, concurrency);
        }
    }

    pub(super) fn handle_action_switch_demo_lang(&mut self, target_lang: &str) {
        let demo_assets = super::demo_bundle::resolve_demo_bundle(target_lang);
        let active_idx = self.state.document.active_doc_idx;

        for asset in demo_assets {
            let virtual_path = PathBuf::from(&asset.virtual_path);
            for doc in self.state.document.open_documents.iter_mut() {
                if doc.path == virtual_path {
                    doc.buffer = asset.content.to_string();
                }
            }
        }
        
        if let Some(idx) = active_idx
            && let Some(doc) = self.state.document.open_documents.get(idx)
            && doc.path.to_string_lossy().starts_with("Katana://Demo/")
        {
            let path = doc.path.clone();
            let src = doc.buffer.clone();
            let concurrency = self.state.config.settings.settings().performance.diagram_concurrency;
            self.full_refresh_preview(&path, &src, false, concurrency);
        }
    }
}
