use crate::shell::KatanaApp;
use crate::views::app_frame::types::*;
use eframe::egui;

impl WorkspaceSidebarItems {
    pub(crate) fn render_workspace_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let ws_icon = if app.state.layout.show_workspace {
            crate::Icon::FolderOpen
        } else {
            crate::Icon::FolderClosed
        };
        let resp = ui.add(
            ws_icon
                .selected_button(
                    ui,
                    crate::icon::IconSize::Large,
                    app.state.layout.show_workspace,
                )
                .sense(egui::Sense::hover()),
        );
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(
                crate::i18n::I18nOps::get()
                    .workspace
                    .workspace_title
                    .clone(),
            );
        if interact_resp.clicked() {
            app.pending_action = crate::app_state::AppAction::ToggleWorkspace;
        }
        Some(interact_resp)
    }

    pub(crate) fn render_search_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let resp = ui.add(
            crate::Icon::Search
                .selected_button(
                    ui,
                    crate::icon::IconSize::Large,
                    app.state.layout.show_search_modal,
                )
                .sense(egui::Sense::hover()),
        );
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(crate::i18n::I18nOps::get().search.modal_title.clone());
        interact_resp.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                true,
                crate::i18n::I18nOps::get().search.modal_title.clone(),
            )
        });
        if interact_resp.clicked() {
            app.pending_action = crate::app_state::AppAction::ToggleSearchModal;
        }
        Some(interact_resp)
    }

    pub(crate) fn render_settings_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let resp = ui.add(
            crate::Icon::Settings
                .selected_button(
                    ui,
                    crate::icon::IconSize::Large,
                    app.state.layout.show_settings,
                )
                .sense(egui::Sense::hover()),
        );
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(crate::i18n::I18nOps::get().settings.title.clone());
        if interact_resp.clicked() {
            app.pending_action = crate::app_state::AppAction::ToggleSettings;
        }
        Some(interact_resp)
    }

    pub(crate) fn render_history_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
        _idx: usize,
    ) -> Option<egui::Response> {
        let is_open = app.state.layout.show_workspace_history_modal;
        let recent_paths = app.state.config.settings.settings().workspace.paths.clone();

        let hover_text = crate::i18n::I18nOps::get()
            .workspace
            .recent_workspaces
            .clone();
        let resp = ui.add_enabled(
            !recent_paths.is_empty(),
            crate::Icon::Document
                .selected_button(ui, crate::icon::IconSize::Large, is_open)
                .sense(egui::Sense::hover()),
        );

        let interact_resp = ui.interact(resp.rect, interact_id, egui::Sense::click_and_drag());
        let interact_resp = if recent_paths.is_empty() {
            interact_resp.on_disabled_hover_text(hover_text)
        } else {
            interact_resp.on_hover_text(hover_text)
        };

        if interact_resp.clicked() && !recent_paths.is_empty() {
            app.pending_action = crate::app_state::AppAction::ToggleWorkspaceHistoryModal;
        }

        Some(interact_resp)
    }
}
