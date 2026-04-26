#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::app::DocumentOps;
    use crate::shell::KatanaApp;
    use katana_core::workspace::{TreeEntry, Workspace};
    use std::path::PathBuf;
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
    fn format_markdown_file_saves_active_buffer_and_refreshes_diagnostics() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        std::fs::write(&path, "# Title").unwrap();

        app.handle_select_document(path.clone(), true);
        app.handle_action_format_markdown_file(path.clone());

        let doc = app.state.active_document().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# Title\n");
        assert_eq!(doc.buffer, "# Title\n");
        assert!(!doc.is_dirty);
        assert!(app.state.diagnostics.get_file_diagnostics(&path).is_empty());
    }

    #[test]
    fn workspace_formatting_skips_ignored_infrastructure_directories() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let visible_path = dir.path().join("visible.md");
        let ignored_dir = dir.path().join(".git");
        let ignored_path = ignored_dir.join("ignored.md");
        std::fs::create_dir_all(&ignored_dir).unwrap();
        std::fs::write(&visible_path, "# Visible").unwrap();
        std::fs::write(&ignored_path, "# Ignored").unwrap();
        app.state.workspace.data = Some(Workspace::new(
            dir.path(),
            vec![
                TreeEntry::File {
                    path: visible_path.clone(),
                },
                TreeEntry::Directory {
                    path: ignored_dir,
                    children: vec![TreeEntry::File {
                        path: ignored_path.clone(),
                    }],
                },
            ],
        ));

        app.handle_action_format_workspace_markdown(PathBuf::from(dir.path()));

        assert_eq!(
            std::fs::read_to_string(&visible_path).unwrap(),
            "# Visible\n"
        );
        assert_eq!(std::fs::read_to_string(&ignored_path).unwrap(), "# Ignored");
    }

    #[test]
    fn workspace_formatting_respects_configured_ignored_directories() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let ignored_dir = dir.path().join("generated");
        let ignored_path = ignored_dir.join("ignored.md");
        std::fs::create_dir_all(&ignored_dir).unwrap();
        std::fs::write(&ignored_path, "# Ignored").unwrap();
        app.state
            .config
            .settings
            .settings_mut()
            .workspace
            .ignored_directories = vec!["generated".to_string()];
        app.state.workspace.data = Some(Workspace::new(
            dir.path(),
            vec![TreeEntry::Directory {
                path: ignored_dir,
                children: vec![TreeEntry::File {
                    path: ignored_path.clone(),
                }],
            }],
        ));

        app.handle_action_format_workspace_markdown(PathBuf::from(dir.path()));

        assert_eq!(std::fs::read_to_string(&ignored_path).unwrap(), "# Ignored");
    }

    #[test]
    fn format_non_markdown_file_reports_failure() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.txt");
        std::fs::write(&path, "plain").unwrap();

        app.handle_action_format_markdown_file(path);

        let Some((message, status_type)) = app.state.layout.status_message else {
            panic!("status message should be set");
        };
        assert_eq!(status_type, crate::app_state::StatusType::Warning);
        assert!(message.contains("note.txt"));
    }
}
