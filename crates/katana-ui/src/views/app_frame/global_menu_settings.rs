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
            for (code, label) in Self::language_options(context) {
                context.select_language(ui, code, &label);
            }
        });
    }

    fn language_options(
        context: &GlobalMenuContext<'_>,
    ) -> [(&'static str, String); LANGUAGE_OPTION_COUNT] {
        [
            ("en", context.i18n().menu.language_en.clone()),
            ("ja", context.i18n().menu.language_ja.clone()),
            ("zh-CN", context.i18n().menu.language_zh_cn.clone()),
            ("zh-TW", context.i18n().menu.language_zh_tw.clone()),
            ("ko", context.i18n().menu.language_ko.clone()),
            ("pt", context.i18n().menu.language_pt.clone()),
            ("fr", context.i18n().menu.language_fr.clone()),
            ("de", context.i18n().menu.language_de.clone()),
            ("es", context.i18n().menu.language_es.clone()),
            ("it", context.i18n().menu.language_it.clone()),
        ]
    }
}

const LANGUAGE_OPTION_COUNT: usize = 10;
