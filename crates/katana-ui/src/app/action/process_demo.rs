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

    pub(super) fn handle_action_open_welcome_screen(&mut self) {
        let lang = self.state.config.settings.settings().language.clone();
        if let Some(asset) = super::demo_bundle::resolve_single_asset(&lang, "welcome.md") {
            self.open_special_virtual_asset(asset);
        }
    }

    /// Handler for AppAction::OpenUserGuide
    pub(super) fn handle_action_open_user_guide(&mut self) {
        let lang = self.state.config.settings.settings().language.clone();
        if let Some(asset) = super::demo_bundle::resolve_single_asset(&lang, "guide.md") {
            self.open_special_virtual_asset(asset);
        }
    }

    pub(super) fn handle_action_switch_demo_lang(&mut self, target_lang: &str) {
        let mut all_assets = super::demo_bundle::resolve_demo_bundle(target_lang);
        if let Some(asset) = super::demo_bundle::resolve_single_asset(target_lang, "welcome.md") {
            all_assets.push(asset);
        }
        if let Some(asset) = super::demo_bundle::resolve_single_asset(target_lang, "guide.md") {
            all_assets.push(asset);
        }

        let active_idx = self.state.document.active_doc_idx;
        let concurrency = self
            .state
            .config
            .settings
            .settings()
            .performance
            .resolved_diagram_concurrency();
        let mut refresh_targets = Vec::new();
        for asset in all_assets {
            let virtual_path = PathBuf::from(&asset.virtual_path);
            for doc in self.state.document.open_documents.iter_mut() {
                if doc.path == virtual_path {
                    doc.buffer = asset.content.to_string();
                    /* WHY: Refresh the preview for all open demo documents to keep tabs synchronized. */
                    refresh_targets.push((doc.path.clone(), doc.buffer.clone()));
                }
            }
        }

        for (path, buffer) in refresh_targets {
            self.full_refresh_preview(&path, &buffer, false, concurrency);
        }

        if let Some(idx) = active_idx
            && let Some(doc) = self.state.document.open_documents.get(idx)
            && doc.path.to_string_lossy().starts_with("Katana://")
        {
            let path = doc.path.clone();
            let src = doc.buffer.clone();
            self.full_refresh_preview(&path, &src, false, concurrency);
        }
    }
}
