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

        egui::TopBottomPanel::top("app_global_menu_bar").show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.file, |ui| {
                    if ui.button(&i18n.menu.open_workspace).clicked() {
                        app.pending_action = AppAction::PickOpenWorkspace;
                        ui.close_menu();
                    }
                    if ui.button(&i18n.menu.save).clicked() {
                        app.pending_action = AppAction::SaveDocument;
                        ui.close_menu();
                    }
                });

                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.view, |ui| {
                    if ui.button(&i18n.menu.command_palette).clicked() {
                        app.pending_action = AppAction::ToggleCommandPalette;
                        ui.close_menu();
                    }
                });

                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.settings, |ui| {
                    if ui.button(&i18n.menu.settings).clicked() {
                        app.pending_action = AppAction::ToggleSettings;
                        ui.close_menu();
                    }
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
                            let is_selected = current_lang == code
                                || (current_lang.is_empty()
                                    && katana_platform::OsLocaleOps::get_default_language()
                                        .as_deref()
                                        == Some(code));
                            if ui.radio(is_selected, label).clicked() {
                                app.pending_action = AppAction::ChangeLanguage(code.to_string());
                                ui.close_menu();
                            }
                        }
                    });
                });

                crate::widgets::MenuButtonOps::show(ui, &i18n.menu.help, |ui| {
                    if ui.button(&i18n.menu.check_updates).clicked() {
                        app.pending_action = AppAction::CheckForUpdates;
                        ui.close_menu();
                    }
                    if ui.button(&i18n.menu.release_notes).clicked() {
                        app.pending_action = AppAction::ShowReleaseNotes;
                        ui.close_menu();
                    }
                    if ui.button(&i18n.menu.demo).clicked() {
                        app.pending_action = AppAction::OpenHelpDemo;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(&i18n.menu.about).clicked() {
                        app.show_about = !app.show_about;
                        ui.close_menu();
                    }
                });
            });
        });
    }
}
