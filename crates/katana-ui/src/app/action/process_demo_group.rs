use crate::app::preview::PreviewOps;
use crate::shell::KatanaApp;
use std::path::PathBuf;

impl KatanaApp {
    pub(super) fn open_special_virtual_asset(&mut self, asset: super::demo_bundle::DemoAsset) {
        let path = PathBuf::from(asset.virtual_path);

        /* WHY: Check if already open */
        let mut found_idx = None;
        for (i, doc) in self.state.document.open_documents.iter_mut().enumerate() {
            if doc.path == path {
                /* WHY: Update reference state */
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

        self.state.document.active_doc_idx = Some(idx);
        let active_path = self.state.document.open_documents[idx].path.clone();
        let src = self.state.document.open_documents[idx].buffer.clone();
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .resolved_diagram_concurrency();
        self.full_refresh_preview(&active_path, &src, false, concurrency);
    }

    pub(super) fn open_demo_group(&mut self, demo_assets: Vec<super::demo_bundle::DemoAsset>) {
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

            /* WHY: Force specific view modes for certain demos */
            if asset.virtual_path.ends_with("lint-fix.md") {
                self.state
                    .document
                    .tab_view_modes
                    .push(crate::state::document::TabViewMode {
                        path: path.clone(),
                        mode: crate::state::document::ViewMode::CodeOnly,
                    });
            }

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
                .resolved_diagram_concurrency();
            self.full_refresh_preview(&path, &src, false, concurrency);
        }
    }
}
