use crate::settings::types::SUBSECTION_SPACING;

pub(crate) struct WorkspaceModalOps;

impl WorkspaceModalOps {
    pub(crate) fn render_no_extension_warning_modal(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        workspace_msgs: &crate::i18n::SettingsWorkspaceMessages,
    ) {
        let modal_id = egui::Id::new("show_no_extension_warning");
        let show_modal = ui.data(|d| d.get_temp::<bool>(modal_id).unwrap_or(false));
        if show_modal {
            let mut close = false;
            let mut confirm = false;
            egui::Window::new(&workspace_msgs.no_extension_warning_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.label(&workspace_msgs.no_extension_warning);
                    ui.add_space(SUBSECTION_SPACING);
                    crate::widgets::AlignCenter::new()
                        .shrink_to_fit(true)
                        .content(|ui| {
                            if ui
                                .button(crate::i18n::I18nOps::get().action.cancel.clone())
                                .clicked()
                            {
                                close = true;
                            }
                            if ui
                                .button(crate::i18n::I18nOps::get().action.confirm.clone())
                                .clicked()
                            {
                                confirm = true;
                                close = true;
                            }
                        })
                        .show(ui);
                });

            let should_close = close || ui.input(|i| i.key_pressed(egui::Key::Escape));

            if confirm
                && !state
                    .config
                    .settings
                    .settings()
                    .workspace
                    .visible_extensions
                    .contains(&"".to_string())
            {
                state
                    .config
                    .settings
                    .settings_mut()
                    .workspace
                    .visible_extensions
                    .push("".to_string());
                let _ = state.config.try_save_settings();
            }

            if should_close {
                ui.data_mut(|d| d.insert_temp(modal_id, false));
            }
        }
    }
}
