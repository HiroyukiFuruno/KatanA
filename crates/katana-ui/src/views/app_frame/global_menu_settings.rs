use super::global_menu_context::GlobalMenuContext;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct GlobalSettingsMenu;

impl GlobalSettingsMenu {
    pub(super) fn render(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let menu_label = context.i18n().menu.settings.clone();
        crate::widgets::MenuButtonOps::show(ui, &menu_label, |ui| {
            context.shortcut_action_item(
                ui,
                "app.settings",
                &menu_label,
                "open_settings",
                AppAction::ToggleSettings,
            );
            ui.separator();
            Self::render_language_menu(ui, context);
        });
    }

    fn render_language_menu(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let menu_label = context.i18n().menu.language.clone();
        crate::widgets::MenuButtonOps::show(ui, &menu_label, |ui| {
            for option in crate::language_options::LanguageOptionOps::menu_options(context.i18n()) {
                context.select_language(ui, option.code, &option.label);
            }
        });
    }
}
