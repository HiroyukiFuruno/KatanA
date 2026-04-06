use crate::shell::KatanaApp;
use crate::views::app_frame::types::*;
use eframe::egui;

impl ExplorerSidebarItems {
    pub(crate) fn render_add_workspace(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let resp = ui.add(
            crate::Icon::Plus
                .button(ui, crate::icon::IconSize::Large)
                .sense(egui::Sense::hover()),
        );
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(
                crate::i18n::I18nOps::get()
                    .workspace
                    .open_workspace_button
                    .clone(),
            );
        if interact_resp.clicked() {
            app.pending_action = crate::shell_ui::ShellUiOps::pick_open_workspace();
        }
        Some(interact_resp)
    }

    pub(crate) fn render_workspace_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let resp = ui.add(
            crate::Icon::FolderOpen
                .selected_button(
                    ui,
                    crate::icon::IconSize::Large,
                    app.state.layout.show_workspace_panel,
                )
                .sense(egui::Sense::hover()),
        );
        app.state.layout.workspace_toggle_y = resp.rect.top();
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(
                crate::i18n::I18nOps::get()
                    .workspace
                    .sidebar_workspace_tooltip
                    .clone(),
            );
        if interact_resp.clicked() {
            app.pending_action = crate::app_state::AppAction::ToggleWorkspacePanel;
        }
        Some(interact_resp)
    }

    pub(crate) fn render_explorer_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let icon = crate::Icon::Explorer;
        let resp = ui.add(
            icon.selected_button(
                ui,
                crate::icon::IconSize::Large,
                app.state.layout.show_explorer,
            )
            .sense(egui::Sense::hover()),
        );
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(crate::i18n::I18nOps::get().workspace.explorer_title.clone());

        interact_resp.context_menu(|ui| {
            if ui
                .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
                .clicked()
            {
                app.pending_action = crate::shell_ui::ShellUiOps::pick_open_workspace();
                ui.close();
            }

            if app.state.workspace.data.is_some() {
                ui.separator();
                if ui
                    .button(crate::i18n::I18nOps::get().menu.close_workspace.clone())
                    .clicked()
                {
                    app.pending_action = crate::app_state::AppAction::CloseWorkspace;
                    ui.close();
                }
            }
        });

        if interact_resp.clicked() {
            app.pending_action = crate::app_state::AppAction::ToggleExplorer;
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
        let is_open = app.state.layout.show_history_panel;
        let recent_paths = app.state.global_workspace.state().histories.clone();

        let hover_text = crate::i18n::I18nOps::get()
            .workspace
            .sidebar_history_tooltip
            .clone();
        let resp = ui.add_enabled(
            !recent_paths.is_empty(),
            crate::Icon::History
                .selected_button(ui, crate::icon::IconSize::Large, is_open)
                .sense(egui::Sense::hover()),
        );
        app.state.layout.history_toggle_y = resp.rect.top();

        let interact_resp = ui.interact(resp.rect, interact_id, egui::Sense::click_and_drag());
        let interact_resp = if recent_paths.is_empty() {
            interact_resp.on_disabled_hover_text(hover_text)
        } else {
            interact_resp.on_hover_text(hover_text)
        };

        if interact_resp.clicked() && !recent_paths.is_empty() {
            app.pending_action = crate::app_state::AppAction::ToggleHistoryPanel;
        }

        Some(interact_resp)
    }
}
