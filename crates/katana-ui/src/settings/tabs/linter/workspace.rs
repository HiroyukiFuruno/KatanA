use crate::i18n::LinterTranslations;
use crate::settings::SETTINGS_TOGGLE_SPACING;
use eframe::egui;

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
            egui::RichText::new(&msgs.advanced_workspace_settings).strong(),
            |ui| {
                let mut use_workspace = state
                    .config
                    .settings
                    .settings()
                    .linter
                    .use_workspace_local_config;

                let workspace_json_path = state
                    .workspace
                    .data
                    .as_ref()
                    .map(|w| w.root.join(".markdownlint.json"));

                if workspace_json_path.is_some() {
                    if ui
                        .add(
                            crate::widgets::LabeledToggle::new(
                                &msgs.use_workspace_local_config,
                                &mut use_workspace,
                            )
                            .position(crate::widgets::TogglePosition::Right)
                            .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                        )
                        .changed()
                    {
                        state
                            .config
                            .settings
                            .settings_mut()
                            .linter
                            .use_workspace_local_config = use_workspace;
                        let _ = state.config.try_save_settings();

                        let target_path =
                            crate::linter_config_bridge::MarkdownLinterConfigOps::target_config_path(
                                state,
                            );

                        /* WHY: If switching and the target file exists, automatically open the advanced settings */
                        if target_path.exists() {
                            *is_advanced_open = true;
                        }
                    }
                    ui.add_space(SETTINGS_TOGGLE_SPACING);
                }

                let json_path =
                    crate::linter_config_bridge::MarkdownLinterConfigOps::target_config_path(state);

                if json_path.exists() {
                    ui.label(&msgs.workspace_has_config);
                    ui.add_space(SETTINGS_TOGGLE_SPACING);
                    if ui
                        .add(
                            egui::Button::new(&crate::i18n::I18nOps::get().common.advanced_settings)
                                .frame_when_inactive(true),
                        )
                        .clicked()
                    {
                        *is_advanced_open = true;
                    }
                } else {
                    ui.label(&msgs.workspace_no_config);
                    ui.add_space(SETTINGS_TOGGLE_SPACING);
                    if ui
                        .add(egui::Button::new(&msgs.create_config).frame_when_inactive(true))
                        .clicked()
                    {
                        if let Some(parent) = json_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if std::fs::write(&json_path, "{\n  \"default\": true\n}\n").is_ok() {
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
}
