use crate::app::action::FileOpenOps;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;
use std::path::PathBuf;

impl KatanaApp {
    pub(super) fn handle_action_pick_open_file(&mut self, ctx: &egui::Context, action: AppAction) {
        if crate::shell_ui::ShellUiOps::is_headless() {
            self.pending_dialog_action = Some(action);
            self.file_dialog.pick_file();
            return;
        }
        let result = crate::shell_ui::ShellUiOps::open_file_dialog_result(
            &FileOpenOps::dialog_extensions(self),
        );
        self.handle_pick_open_file_result(ctx, result, action);
    }

    pub(crate) fn handle_pick_open_file_result(
        &mut self,
        ctx: &egui::Context,
        result: crate::shell_ui::NativeDialogResult<PathBuf>,
        action: AppAction,
    ) {
        match result {
            crate::shell_ui::NativeDialogResult::Picked(path) => {
                match action {
                    AppAction::PickOpenFileInNewWorkspace => {
                        FileOpenOps::open_as_temporary_workspace(self, path.clone());
                    }
                    AppAction::PickOpenFileInCurrentWorkspace => {
                        FileOpenOps::open_in_current_workspace(self, path.clone());
                    }
                    _ => return,
                }
                let active = self
                    .state
                    .active_document()
                    .is_some_and(|document| document.path == path);
                let pending = self.pending_workspace_file_open.as_ref() == Some(&path);
                if active || pending {
                    crate::app::preview::keyboard_focus::PageNavigationFocusOps::arm_for_selection(
                        ctx, &path,
                    );
                } else {
                    crate::app::preview::keyboard_focus::PageNavigationFocusOps::clear(ctx);
                }
            }
            crate::shell_ui::NativeDialogResult::Cancelled => {}
            crate::shell_ui::NativeDialogResult::Unavailable => {
                self.pending_dialog_action = Some(action);
                self.file_dialog.pick_file();
            }
        }
    }
}
