/* WHY: Isolated file system operation modals (Create/Rename/Delete) for clean state handling. */

use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(crate) fn show_file_operations_modals(&mut self, ctx: &egui::Context) {
        if let Some(mut modal_data) = self.state.layout.create_fs_node_modal.take() {
            let visible_ext = self
                .state
                .config
                .settings
                .settings()
                .workspace
                .visible_extensions
                .clone();
            let close = crate::views::modals::file_ops::CreateFsNodeModal::new(
                &mut modal_data,
                &visible_ext,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.create_fs_node_modal = Some(modal_data);
            }
        }
        if let Some(mut modal_data) = self.state.layout.rename_modal.take()
            && !crate::views::modals::file_ops::RenameModal::new(
                &mut modal_data,
                &mut self.pending_action,
            )
            .show(ctx)
        {
            self.state.layout.rename_modal = Some(modal_data);
        }
        if let Some(modal_data) = self.state.layout.delete_modal.take()
            && !crate::views::modals::file_ops::DeleteModal::new(
                &modal_data,
                &mut self.pending_action,
            )
            .show(ctx)
        {
            self.state.layout.delete_modal = Some(modal_data);
        }
    }
}
