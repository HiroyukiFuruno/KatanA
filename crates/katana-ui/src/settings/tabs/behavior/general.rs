use crate::app_state::{AppAction, AppState};
use crate::settings::*;
use eframe::egui;

pub struct BehaviorGeneralOps;

impl BehaviorGeneralOps {
    pub(super) fn render(ui: &mut egui::Ui, state: &mut AppState) -> Option<AppAction> {
        let i18n = crate::i18n::I18nOps::get();
        ui.label(egui::RichText::new(&i18n.settings.general).strong());
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let current = state.config.settings.settings().language.clone();
        let selected = crate::language_options::LanguageOptionOps::label_for(&current, i18n);
        let mut action = None;

        ui.indent("behavior_general_settings_body", |ui| {
            crate::widgets::AlignCenter::new()
                .left(|ui| ui.label(&i18n.menu.language))
                .right(|ui| {
                    egui::ComboBox::from_id_salt("settings_language_selector")
                        .selected_text(selected)
                        .width(LANGUAGE_SELECTOR_WIDTH)
                        .show_ui(ui, |ui| {
                            for option in
                                crate::language_options::LanguageOptionOps::menu_options(i18n)
                            {
                                if ui
                                    .add(
                                        egui::Button::selectable(
                                            current == option.code,
                                            option.label,
                                        )
                                        .frame_when_inactive(true),
                                    )
                                    .clicked()
                                {
                                    action =
                                        Some(AppAction::ChangeLanguage(option.code.to_string()));
                                }
                            }
                        })
                        .response
                })
                .show(ui);
        });

        action
    }
}

const LANGUAGE_SELECTOR_WIDTH: f32 = 240.0;
