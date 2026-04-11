use crate::shell::KatanaApp;
use crate::views::app_frame::types::*;
use eframe::egui;

impl ExplorerSidebarItems {
    /* WHY: Renders the settings toggle button with selection state and i18n tooltip. */
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

    /* WHY: Renders the history toggle button with availability check and i18n tooltip. */
    pub(crate) fn render_history_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
        _idx: usize,
    ) -> Option<egui::Response> {
        let recent_paths = app.state.global_workspace.state().histories.clone();

        let hover_text = crate::i18n::I18nOps::get()
            .workspace
            .sidebar_history_tooltip
            .clone();
        let resp = ui.add_enabled(
            !recent_paths.is_empty(),
            crate::Icon::History
                .selected_button(
                    ui,
                    crate::icon::IconSize::Large,
                    app.state.layout.active_rail_popup
                        == Some(crate::state::layout::RailPopup::History),
                )
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
            app.pending_action = crate::app_state::AppAction::ToggleRailPopup(
                crate::state::layout::RailPopup::History,
            );
        }

        Some(interact_resp)
    }

    /* WHY: Renders the help toggle button with i18n support and shortcut info. */
    pub(crate) fn render_help_toggle(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        interact_id: egui::Id,
    ) -> Option<egui::Response> {
        let resp = ui.add(
            crate::Icon::Help
                .selected_button(
                    ui,
                    crate::icon::IconSize::Large,
                    app.state.layout.active_rail_popup
                        == Some(crate::state::layout::RailPopup::Help),
                )
                .sense(egui::Sense::hover()),
        );
        let interact_resp = ui
            .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
            .on_hover_text(crate::i18n::I18nOps::get().menu.help.clone());
        if interact_resp.clicked() {
            app.pending_action =
                crate::app_state::AppAction::ToggleRailPopup(crate::state::layout::RailPopup::Help);
        }
        Some(interact_resp)
    }
}
