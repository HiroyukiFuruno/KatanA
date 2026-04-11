use super::content::ExplorerContent;
use super::header::ExplorerHeader;
use crate::app_state::AppAction;
use crate::shell_ui::{
    WORKSPACE_SPINNER_INNER_MARGIN, WORKSPACE_SPINNER_OUTER_MARGIN, WORKSPACE_SPINNER_TEXT_MARGIN,
};
use eframe::egui;

pub(crate) struct ExplorerPanel<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub histories: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
    pub action: &'a mut AppAction,
}

impl<'a> ExplorerPanel<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        histories: &'a [String],
        active_path: Option<&'a std::path::Path>,
        tab_groups: &'a [crate::state::document::TabGroup],
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            histories,
            active_path,
            tab_groups,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let workspace = self.workspace;
        let search = self.search;
        let histories = self.histories;
        let active_path = self.active_path;
        let tab_groups = self.tab_groups;
        let action = self.action;
        let panel_width = ui.available_width();
        ui.set_max_width(panel_width);
        ui.set_min_width(panel_width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

        let is_loading = workspace.is_loading;

        ExplorerHeader::new(workspace, search, action).show(ui);

        ui.separator();

        if is_loading {
            ui.add_space(WORKSPACE_SPINNER_OUTER_MARGIN);
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.add_space(WORKSPACE_SPINNER_INNER_MARGIN);
                    ui.spinner();
                    ui.add_space(WORKSPACE_SPINNER_TEXT_MARGIN);
                    ui.label(crate::i18n::I18nOps::get().action.refresh_explorer.clone());
                })
                .show(ui);
        } else {
            ExplorerContent::new(
                workspace,
                search,
                histories,
                active_path,
                tab_groups,
                action,
            )
            .show(ui);
        }
    }
}
