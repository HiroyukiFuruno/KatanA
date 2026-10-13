use crate::app::action::FileOpenOps;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

pub(crate) struct DroppedFileOps;

impl DroppedFileOps {
    pub(crate) fn queue(app: &mut KatanaApp, ctx: &egui::Context) {
        let dropped_files = FileOpenOps::dropped_openable_file_paths(app, ctx);
        if dropped_files.is_empty() || !matches!(app.pending_action, AppAction::None) {
            return;
        }
        app.pending_action = AppAction::OpenDroppedFiles(dropped_files);
    }
}
