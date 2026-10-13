use super::types::*;
use crate::shell::KatanaApp;

impl<'a> WorkspaceToolbar<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        let workspace_tabs = app
            .state
            .global_workspace
            .state()
            .open_workspace_tabs
            .clone();
        if workspace_tabs.is_empty() {
            return;
        }
        let active_workspace = app.state.global_workspace.state().active_workspace.clone();
        let action = crate::views::top_bar::WorkspaceTabBar::new(
            &workspace_tabs,
            active_workspace.as_deref(),
            &mut app.state.workspace.scroll_to_workspace_tab,
        )
        .show(ui);
        if let Some(action) = action {
            app.pending_action = action;
        }
    }
}
