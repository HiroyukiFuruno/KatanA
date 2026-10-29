use crate::app::WorkspaceOps;
use crate::app::action::FileOpenOps;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(super) fn handle_file_dialog_action(
        &mut self,
        ctx: &egui::Context,
        path: std::path::PathBuf,
        action: AppAction,
    ) {
        let selected_path = path.clone();
        let is_document_open = matches!(
            &action,
            AppAction::PickOpenFileInCurrentWorkspace | AppAction::PickOpenFileInNewWorkspace
        );
        match action {
            AppAction::PickOpenWorkspace => {
                self.handle_open_explorer(path);
            }
            AppAction::PickOpenFileInCurrentWorkspace => {
                FileOpenOps::open_in_current_workspace(self, path);
            }
            AppAction::PickOpenFileInNewWorkspace => {
                FileOpenOps::open_as_temporary_workspace(self, path);
            }
            AppAction::IngestImageFile => {
                let Ok(bytes) = std::fs::read(&path) else {
                    return;
                };
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png");
                self.process_image_ingest(&bytes, ext);
            }
            AppAction::PickExportDocument {
                doc_path,
                ext,
                source,
            } => {
                use crate::app::export_poll::ExportPoll;
                self.perform_tool_export(&source, &ext, path, &doc_path);
            }
            _ => {}
        }
        if is_document_open {
            let active_path = self
                .state
                .active_document()
                .map(|document| document.path.as_path());
            crate::app::preview::keyboard_focus::PageNavigationFocusOps::finish_file_dialog(
                ctx,
                &selected_path,
                active_path,
                self.pending_workspace_file_open.as_deref(),
            );
        }
    }
}
