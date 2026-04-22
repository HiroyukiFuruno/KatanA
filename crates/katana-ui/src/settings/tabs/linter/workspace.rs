use crate::app_state::AppAction;
use crate::i18n::LinterTranslations;
use crate::settings::SETTINGS_TOGGLE_SPACING;
use eframe::egui;

pub(crate) struct WorkspaceSettingsOps;

impl WorkspaceSettingsOps {
    pub(crate) fn render_workspace_settings(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        msgs: &LinterTranslations,
        action: &mut Option<AppAction>,
    ) {
        let mut use_workspace = state
            .config
            .settings
            .settings()
            .linter
            .use_workspace_local_config;

        let global_config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("KatanA");
        let global_json_path = global_config_dir.join(".markdownlint.json");

        let workspace_json_path = state
            .workspace
            .data
            .as_ref()
            .map(|w| w.root.join(".markdownlint.json"));

        if let Some(ws_path) = &workspace_json_path {
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

                let target_path = if use_workspace {
                    ws_path.clone()
                } else {
                    global_json_path.clone()
                };

                /* WHY: If switching and the target file exists, automatically open it */
                if target_path.exists() {
                    state.layout.show_settings = false;
                    *action = Some(AppAction::SelectDocument(target_path));
                }
            }
            ui.add_space(SETTINGS_TOGGLE_SPACING);
        }

        let json_path = if use_workspace {
            workspace_json_path.unwrap_or(global_json_path)
        } else {
            global_json_path
        };

        crate::widgets::AlignCenter::new()
            .content(|ui| {
                if json_path.exists() {
                    ui.label(&msgs.workspace_has_config);
                    if ui
                        .add(egui::Button::new(&msgs.open_config).frame_when_inactive(true))
                        .clicked()
                    {
                        state.layout.show_settings = false;
                        *action = Some(AppAction::SelectDocument(json_path));
                    }
                } else {
                    ui.label(&msgs.workspace_no_config);
                    if ui
                        .add(egui::Button::new(&msgs.create_config).frame_when_inactive(true))
                        .clicked()
                    {
                        if let Some(parent) = json_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if std::fs::write(&json_path, "{\n  \"default\": true\n}\n").is_ok() {
                            state.layout.show_settings = false;
                            *action = Some(AppAction::SelectDocument(json_path));
                        }
                    }
                }
            })
            .show(ui);
    }
}
