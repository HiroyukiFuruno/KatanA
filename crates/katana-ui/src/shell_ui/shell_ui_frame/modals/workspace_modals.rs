/* WHY: Isolated workspace and history panel orchestration for clean lifecycle management. */

use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(crate) fn show_workspace_visibility_modals(&mut self, ctx: &egui::Context) {
        if self.state.layout.show_workspace_panel {
            let current_root = self
                .state
                .workspace
                .data
                .as_ref()
                .map(|ws| ws.root.display().to_string());
            let title = &crate::i18n::I18nOps::get()
                .workspace
                .sidebar_workspace_tooltip;
            crate::views::modals::workspace_toggle::WorkspaceToggleModal::new(
                title,
                self.state.global_workspace.state().persisted.as_slice(),
                current_root,
                &mut self.pending_action,
                &mut self.state.layout.show_workspace_panel,
                false,
                self.state.layout.workspace_toggle_y,
            )
            .show(ctx);
        }

        if self.state.layout.show_history_panel {
            let current_root = self
                .state
                .workspace
                .data
                .as_ref()
                .map(|ws| ws.root.display().to_string());
            let title = &crate::i18n::I18nOps::get()
                .workspace
                .workspace_history_title;
            crate::views::modals::workspace_toggle::WorkspaceToggleModal::new(
                title,
                self.state.global_workspace.state().histories.as_slice(),
                current_root,
                &mut self.pending_action,
                &mut self.state.layout.show_history_panel,
                true,
                self.state.layout.history_toggle_y,
            )
            .show(ctx);
        }
    }
}
