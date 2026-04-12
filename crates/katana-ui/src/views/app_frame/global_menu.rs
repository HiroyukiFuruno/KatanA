use super::types::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl<'a> GlobalMenuBar<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        /* WHY: Only render on platforms that do not have a native global menu. */
        if katana_platform::PlatformContractOps::has_native_global_menu() {
            /* WHY: macOS uses native menu (app delegate) */
            return;
        }

        let app = self.app;
        let i18n = crate::i18n::I18nOps::get();

        let is_avail = |id: &str, state: &crate::app_state::AppState| {
            crate::state::command_inventory::CommandInventory::all()
                .iter()
                .find(|c| c.id == id)
                .is_some_and(|c| (c.is_available)(state))
        };

        macro_rules! btn {
            ($ui:expr, $id:expr, $label:expr, $action:expr) => {
                if $ui
                    .add_enabled(is_avail($id, &app.state), egui::Button::new($label))
                    .clicked()
                {
                    app.pending_action = $action;
                    $ui.close_menu();
                }
            };
            ($ui:expr, $id:expr, $label:expr, $shortcut:expr, $action:expr) => {
                if $ui
                    .add_enabled(
                        is_avail($id, &app.state),
                        egui::Button::new($label)
                            .shortcut_text(crate::os_command::OsCommandOps::get($shortcut)),
                    )
                    .clicked()
                {
                    app.pending_action = $action;
                    $ui.close_menu();
                }
            };
        }

        egui::TopBottomPanel::top("app_global_menu_bar").show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                crate::widgets::MenuButtonOps::show(ui, "KatanA", |ui| {
                    btn!(
                        ui,
                        "app.settings",
                        &i18n.menu.settings,
                        "open_settings",
                        AppAction::ToggleSettings
                    );
                    crate::widgets::MenuButtonOps::show(ui, &i18n.menu.language, |ui| {
                        let current_lang = app.state.config.settings.settings().language.clone();
                        let langs = [
                            ("en", &i18n.menu.language_en),
                            ("ja", &i18n.menu.language_ja),
                            ("zh-CN", &i18n.menu.language_zh_cn),
                            ("zh-TW", &i18n.menu.language_zh_tw),
                            ("ko", &i18n.menu.language_ko),
                            ("pt", &i18n.menu.language_pt),
                            ("fr", &i18n.menu.language_fr),
                            ("de", &i18n.menu.language_de),
                            ("es", &i18n.menu.language_es),
                            ("it", &i18n.menu.language_it),
                        ];
                        for (code, label) in langs {
                            let default = katana_platform::OsLocaleOps::get_default_language();
                            let is_selected = current_lang == code
                                || (current_lang.is_empty() && default.as_deref() == Some(code));
                            if ui.radio(is_selected, label).clicked() {
                                app.pending_action = AppAction::ChangeLanguage(code.to_string());
                                ui.close_menu();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button(&i18n.menu.quit).clicked() {
                        app.pending_action = AppAction::Quit;
                        ui.close_menu();
                    }
                });

                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.file, |ui| {
                    btn!(
                        ui,
                        "file.open_workspace",
                        &i18n.menu.open_workspace,
                        "open_workspace",
                        AppAction::PickOpenWorkspace
                    );
                    btn!(
                        ui,
                        "file.close_workspace",
                        &i18n.menu.close_workspace,
                        AppAction::CloseWorkspace
                    );
                    ui.separator();
                    btn!(
                        ui,
                        "file.save",
                        &i18n.menu.save,
                        "save_document",
                        AppAction::SaveDocument
                    );
                });

                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.view, |ui| {
                    btn!(
                        ui,
                        "view.command_palette",
                        &i18n.menu.command_palette,
                        "open_palette",
                        AppAction::ToggleCommandPalette
                    );
                    ui.separator();
                    btn!(
                        ui,
                        "view.explorer",
                        &i18n.search.command_explorer,
                        "toggle_sidebar",
                        AppAction::ToggleExplorer
                    );
                    btn!(
                        ui,
                        "view.refresh_explorer",
                        &i18n.search.command_refresh_explorer,
                        AppAction::RefreshExplorer
                    );
                    ui.separator();
                    btn!(
                        ui,
                        "view.close_all",
                        &i18n.search.command_close_all,
                        AppAction::CloseAllDocuments
                    );
                });

                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.help, |ui| {
                    btn!(
                        ui,
                        "help.welcome_screen",
                        &i18n.menu.welcome_screen,
                        AppAction::OpenWelcomeScreen
                    );
                    btn!(
                        ui,
                        "help.user_guide",
                        &i18n.menu.user_guide,
                        AppAction::OpenUserGuide
                    );
                    btn!(
                        ui,
                        "help.demo",
                        &i18n.menu.demo,
                        "open_demo",
                        AppAction::OpenHelpDemo
                    );
                    ui.separator();
                    btn!(ui, "help.github", &i18n.menu.github, AppAction::OpenGitHub);
                    btn!(
                        ui,
                        "help.website",
                        &i18n.menu.website,
                        AppAction::OpenOfficialWebsite
                    );
                    ui.separator();
                    btn!(
                        ui,
                        "help.release_notes",
                        &i18n.menu.release_notes,
                        AppAction::ShowReleaseNotes
                    );
                    btn!(
                        ui,
                        "help.check_updates",
                        &i18n.menu.check_updates,
                        AppAction::CheckForUpdates
                    );
                    ui.separator();
                    btn!(ui, "help.about", &i18n.menu.about, AppAction::ToggleAbout);
                });
            });
        });
    }
}
