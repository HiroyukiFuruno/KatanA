use crate::shell::KatanaApp;
use crate::views::app_frame::types::*;
use eframe::egui;

/* WHY: Collection of static sidebar item rendering functions. */
impl ExplorerSidebarItems {
    /* WHY: Renders the "Add Workspace" button with i18n tooltip and picker action. */
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

    /* WHY: Renders the workspace panel visibility toggle button. */
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

    /* WHY: Renders the file explorer panel toggle with context menu support. */
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
            /* WHY: Set cooldown so hover does not re-open immediately when the cursor
             * stays on the button after a click (covers both pin and unpin). */
            interact_resp
                .ctx
                .data_mut(|d| d.insert_temp(egui::Id::new("explorer_hover_cooldown"), true));
        }
        Some(interact_resp)
    }

    /* WHY: Renders the sidebar search toggle button targeting the foreground popup. */
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
            app.state.layout.active_rail_popup = None;
        }
        Some(interact_resp)
    }
}
