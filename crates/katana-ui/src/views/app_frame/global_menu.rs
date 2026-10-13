use super::types::*;
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

        let i18n = crate::i18n::I18nOps::get();
        let mut context = super::global_menu_context::GlobalMenuContext::new(self.app, i18n);

        egui::Panel::top("app_global_menu_bar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                super::global_menu_app::GlobalAppMenu::render(ui, &mut context);
                super::global_menu_file::GlobalFileMenu::render(ui, &mut context);
                super::global_menu_view::GlobalViewMenu::render(ui, &mut context);
                super::global_menu_settings::GlobalSettingsMenu::render(ui, &mut context);
                super::global_menu_help::GlobalHelpMenu::render(ui, &mut context);
            });
        });
    }
}
