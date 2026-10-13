use crate::i18n::LinterTranslations;
use crate::settings::SETTINGS_TOGGLE_SPACING;
use eframe::egui;
use std::path::PathBuf;

#[cfg(test)]
mod workspace_tests;

pub(crate) struct WorkspaceSettingsOps;

impl WorkspaceSettingsOps {
    pub(crate) fn render_workspace_settings(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        is_advanced_open: &mut bool,
        force_open: Option<bool>,
    ) {
        crate::widgets::Accordion::new(
            "linter_workspace_settings_accordion",
            egui::RichText::new(&msgs.workspace_rules).strong(),
            |ui| {
                let Some(workspace_json_path) = Self::workspace_json_path(state) else {
                    ui.label(&msgs.open_workspace_to_configure);
                    return;
                };
                let mut use_workspace = state
                    .config
                    .settings
                    .settings()
                    .linter
                    .use_workspace_local_config;

                if ui
                    .add(
                        crate::widgets::LabeledToggle::new(
                            &msgs.use_workspace_rules,
                            &mut use_workspace,
                        )
                        .position(crate::widgets::TogglePosition::Right)
                        .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                    )
                    .changed()
                {
                    Self::save_workspace_config_toggle(state, use_workspace);
                    ui.data_mut(|data| {
                        data.insert_temp(egui::Id::new("katana_pending_linter_update"), true);
                    });
                }
                ui.add_space(SETTINGS_TOGGLE_SPACING);

                if let Some(config_path) = Self::existing_workspace_config_path(state) {
                    ui.label(&msgs.workspace_rules_found);
                    ui.add_space(SETTINGS_TOGGLE_SPACING);
                    if ui
                        .add(
                            egui::Button::new(&crate::i18n::I18nOps::get().common.advanced_settings)
                                .frame_when_inactive(true),
                        )
                        .clicked()
                    {
                        Self::set_advanced_config_path(ui, config_path);
                        *is_advanced_open = true;
                    }
                } else {
                    ui.label(&msgs.workspace_rules_missing);
                    ui.add_space(SETTINGS_TOGGLE_SPACING);
                    if ui
                        .add(
                            egui::Button::new(&msgs.create_workspace_rules)
                                .frame_when_inactive(true),
                        )
                        .clicked()
                    {
                        if let Some(parent) = workspace_json_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let config =
                            crate::linter_config_bridge::MarkdownLinterConfigOps::load_effective_config(
                                state,
                            );
                        if config.save(&workspace_json_path).is_ok() {
                            Self::save_workspace_config_toggle(state, true);
                            ui.data_mut(|data| {
                                data.insert_temp(
                                    egui::Id::new("katana_pending_linter_update"),
                                    true,
                                );
                            });
                            Self::set_advanced_config_path(ui, workspace_json_path);
                            *is_advanced_open = true;
                        }
                    }
                }
            },
        )
        .default_open(true)
        .force_open(force_open)
        .show(ui);
    }

    pub(super) fn save_workspace_config_toggle(
        state: &mut crate::app_state::AppState,
        use_workspace: bool,
    ) {
        state
            .config
            .settings
            .settings_mut()
            .linter
            .use_workspace_local_config = use_workspace;
        let _ = state.config.try_save_settings();
    }

    fn workspace_json_path(state: &crate::app_state::AppState) -> Option<PathBuf> {
        state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.join(".markdownlint.json"))
    }

    fn workspace_jsonc_path(state: &crate::app_state::AppState) -> Option<PathBuf> {
        state
            .workspace
            .data
            .as_ref()
            .map(|workspace| workspace.root.join(".markdownlint.jsonc"))
    }

    pub(super) fn existing_workspace_config_path(
        state: &crate::app_state::AppState,
    ) -> Option<PathBuf> {
        let json_path = Self::workspace_json_path(state);
        if let Some(path) = json_path.as_ref()
            && path.exists()
        {
            return json_path;
        }

        let jsonc_path = Self::workspace_jsonc_path(state);
        if let Some(path) = jsonc_path.as_ref()
            && path.exists()
        {
            return jsonc_path;
        }

        None
    }

    fn set_advanced_config_path(ui: &mut egui::Ui, path: PathBuf) {
        ui.data_mut(|data| {
            data.insert_temp(egui::Id::new("linter_advanced_config_path"), path);
        });
    }
}
