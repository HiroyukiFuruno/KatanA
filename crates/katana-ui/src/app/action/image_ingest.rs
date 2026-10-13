use crate::shell::*;

impl KatanaApp {
    /// Ingest an image file from the local filesystem.
    pub(crate) fn handle_action_ingest_image_file(&mut self) {
        if crate::shell_ui::ShellUiOps::is_headless() {
            self.pending_dialog_action = Some(crate::app_state::AppAction::IngestImageFile);
            self.file_dialog.pick_file();
            return;
        }

        let file_result = std::panic::catch_unwind(|| {
            rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                .pick_file()
        });

        match file_result {
            Ok(Some(source_path)) => {
                let Ok(bytes) = std::fs::read(&source_path) else {
                    return;
                };
                let ext = source_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("png");
                self.process_image_ingest(&bytes, ext);
            }
            Ok(None) => {}
            Err(_) => {
                self.pending_dialog_action = Some(crate::app_state::AppAction::IngestImageFile);
                self.file_dialog.pick_file();
            }
        }
    }

    /// Ingest an image from the clipboard.
    pub(crate) fn handle_action_ingest_clipboard_image(&mut self) {
        if !super::clipboard_image::ClipboardImageOps::has_image_payload() {
            return;
        }

        match super::clipboard_image::ClipboardImageOps::read_image_payload() {
            Ok(payload) => self.process_image_ingest(&payload.bytes, payload.extension),
            Err(err) => {
                tracing::warn!("Clipboard image ingest failed: {err}");
                self.state.layout.status_message = Some((
                    format!(
                        "{}: {err}",
                        crate::i18n::I18nOps::get()
                            .search
                            .command_ingest_clipboard_image
                    ),
                    crate::app_state::StatusType::Error,
                ));
            }
        }
    }

    /// Process the ingested image bytes: save to asset dir and insert markdown tag.
    pub(crate) fn process_image_ingest(&mut self, source_bytes: &[u8], extension: &str) {
        let settings = self.state.config.settings.settings().clone();

        let Some(doc) = self.state.active_document() else {
            return;
        };

        if doc.path.as_os_str().is_empty() {
            tracing::warn!("Image ingest requires a saved Markdown file.");
            self.state.layout.status_message = Some((
                "Image ingest requires a saved Markdown file.".to_string(),
                crate::app_state::StatusType::Error,
            ));
            return;
        }

        let ingest_config = &settings.ingest;
        let uuid_str = uuid::Uuid::new_v4().to_string();

        let (dest_path, md_path) = resolve_image_ingest_paths(
            &doc.path,
            &ingest_config.image_save_directory,
            &ingest_config.image_name_format,
            extension,
            &uuid_str,
        );

        let save_dir = dest_path.parent().unwrap();
        if ingest_config.create_directory_if_not_exists
            && let Err(e) = std::fs::create_dir_all(save_dir)
        {
            tracing::error!("Failed to create image save directory: {}", e);
            return;
        }

        if let Err(e) = std::fs::write(&dest_path, source_bytes) {
            tracing::error!("Failed to write ingest image: {}", e);
            return;
        }

        let md_text = format!("![]({})", md_path);

        self.handle_action_insert_raw_markdown(&md_text);
        self.pending_action = crate::app_state::AppAction::RefreshExplorer;
    }
}

/// Pure function to calculate save destination and markdown tag path.
fn resolve_image_ingest_paths(
    doc_path: &std::path::Path,
    image_save_directory: &str,
    image_name_format: &str,
    extension: &str,
    uuid_str: &str,
) -> (std::path::PathBuf, String) {
    let base_dir = doc_path.parent().unwrap_or(doc_path);
    let save_dir = base_dir.join(image_save_directory);

    let file_name = image_name_format.replace("{uuid}", uuid_str);
    let final_name = format!("{}.{}", file_name, extension);
    let dest_path = save_dir.join(&final_name);

    let relative_asset_dir = std::path::Path::new(image_save_directory);
    let final_dest_rel = relative_asset_dir.join(&final_name);
    let md_path = final_dest_rel.to_string_lossy().replace("\\", "/");

    (dest_path, md_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::document::DocumentOps;
    use crate::app_state::AppState;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
    use std::path::PathBuf;

    fn make_app() -> KatanaApp {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
            katana_platform::workspace::InMemoryWorkspaceRepository::default(),
        ));
        KatanaApp::new(state)
    }

    #[test]
    fn test_resolve_image_ingest_paths() {
        let doc_path = PathBuf::from("/users/doc/notes.md");
        let (dest, md) =
            resolve_image_ingest_paths(&doc_path, "./asset/img", "img_{uuid}", "png", "1234");

        assert_eq!(dest, PathBuf::from("/users/doc/./asset/img/img_1234.png"));
        assert_eq!(md, "./asset/img/img_1234.png");
    }

    #[test]
    fn test_resolve_image_ingest_paths_absolute() {
        let doc_path = PathBuf::from("/users/doc/notes.md");
        let (dest, md) = resolve_image_ingest_paths(&doc_path, "assets", "{uuid}", "jpg", "1234");

        assert_eq!(dest, PathBuf::from("/users/doc/assets/1234.jpg"));
        assert_eq!(md, "assets/1234.jpg");
    }

    #[test]
    fn process_image_ingest_saves_asset_and_inserts_markdown() {
        let mut app = make_app();
        let temp_dir = tempfile::tempdir().unwrap();
        let doc_path = temp_dir.path().join("note.md");
        std::fs::write(&doc_path, "alpha").unwrap();
        app.handle_select_document(doc_path, true);

        app.process_image_ingest(b"image-bytes", "png");

        let doc = app.state.active_document().unwrap();
        assert!(
            doc.buffer.starts_with("alpha![](./asset/img/"),
            "image ingest should insert markdown at the active document cursor"
        );
        assert!(
            doc.buffer.ends_with(".png)"),
            "image ingest should keep the source extension in markdown"
        );

        let asset_dir = temp_dir.path().join("./asset/img");
        let asset_paths: Vec<_> = std::fs::read_dir(asset_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(asset_paths.len(), 1);
        assert_eq!(std::fs::read(&asset_paths[0]).unwrap(), b"image-bytes");
        assert!(matches!(
            app.pending_action,
            crate::app_state::AppAction::RefreshExplorer
        ));
    }

    #[test]
    #[ignore = "uses the current OS clipboard; run manually while an image is copied"]
    fn live_current_os_clipboard_image_ingest_inserts_markdown() {
        let mut app = make_app();
        let temp_dir = tempfile::tempdir().unwrap();
        let doc_path = temp_dir.path().join("note.md");
        std::fs::write(&doc_path, "alpha").unwrap();
        app.handle_select_document(doc_path, true);

        app.handle_action_ingest_clipboard_image();

        let doc = app.state.active_document().unwrap();
        assert!(
            doc.buffer.contains("![](./asset/img/"),
            "clipboard image ingest should insert markdown, buffer={:?}, status={:?}",
            doc.buffer,
            app.state.layout.status_message
        );
    }
}
