#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::app::DocumentOps;
    use crate::shell::KatanaApp;
    use katana_core::workspace::Workspace;
    use std::sync::Arc;

    fn make_app() -> KatanaApp {
        let state = crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    #[test]
    fn refresh_diagnostics_reloads_linter_config_when_content_is_unchanged() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".markdownlint.json");
        let doc_path = dir.path().join("doc.md");
        std::fs::write(&config_path, r#"{"default": false, "MD001": true}"#).unwrap();
        std::fs::write(&doc_path, "# Title\n### Skipped\n").unwrap();
        app.state.workspace.data = Some(Workspace::new(dir.path(), Vec::new()));
        app.state
            .config
            .settings
            .settings_mut()
            .linter
            .use_workspace_local_config = true;

        app.handle_select_document(doc_path.clone(), true);
        app.handle_action_refresh_diagnostics();
        assert!(
            app.state
                .diagnostics
                .get_file_diagnostics(&doc_path)
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "MD001")
        );

        std::fs::write(&config_path, r#"{"default": false, "MD001": false}"#).unwrap();
        app.handle_action_refresh_diagnostics();

        assert!(
            app.state
                .diagnostics
                .get_file_diagnostics(&doc_path)
                .is_empty()
        );
    }
}
