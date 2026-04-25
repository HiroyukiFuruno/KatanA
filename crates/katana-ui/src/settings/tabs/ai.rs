use super::types::*;
use crate::settings::*;
use eframe::egui;
use katana_platform::OllamaSettings;

const SETTINGS_TEXT_WIDTH: f32 = 360.0;
const TIMEOUT_MIN_SECS: u64 = 1;
const TIMEOUT_MAX_SECS: u64 = 600;

impl AiTabOps {
    pub(crate) fn render_ai_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let messages = &crate::i18n::I18nOps::get().settings.ai;
        let original = state.config.settings.settings().ai.ollama.clone();
        let mut ollama = original.clone();

        Self::render_ollama_connection(ui, &mut ollama, messages);
        ui.add_space(SECTION_SPACING);
        Self::render_capability_settings(ui, &mut ollama, messages);

        if ollama != original {
            state.config.settings.settings_mut().ai.ollama = ollama;
            let _ = state.config.try_save_settings();
            state.refresh_ai_registry_from_settings();
        }
    }

    fn render_ollama_connection(
        ui: &mut egui::Ui,
        ollama: &mut OllamaSettings,
        messages: &crate::i18n::SettingsAiMessages,
    ) {
        SettingsOps::section_header(ui, &messages.provider_section);
        Self::text_row(ui, &messages.endpoint, &mut ollama.endpoint);
        ui.label(
            egui::RichText::new(&messages.endpoint_hint)
                .weak()
                .size(HINT_FONT_SIZE),
        );
        ui.add_space(SUBSECTION_SPACING);
        Self::text_row(ui, &messages.model, &mut ollama.selected_model);
        ui.label(
            egui::RichText::new(&messages.model_hint)
                .weak()
                .size(HINT_FONT_SIZE),
        );
        if ollama.selected_model.trim().is_empty() {
            ui.label(egui::RichText::new(&messages.model_required).strong());
        }
        ui.add_space(SUBSECTION_SPACING);
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                ui.label(&messages.timeout_secs);
                ui.add(
                    egui::DragValue::new(&mut ollama.timeout_secs)
                        .range(TIMEOUT_MIN_SECS..=TIMEOUT_MAX_SECS)
                        .suffix(format!(" {}", messages.timeout_hint)),
                );
            })
            .show(ui);
        ui.label(
            egui::RichText::new(&messages.lightweight_hint)
                .weak()
                .size(HINT_FONT_SIZE),
        );
    }

    fn render_capability_settings(
        ui: &mut egui::Ui,
        ollama: &mut OllamaSettings,
        messages: &crate::i18n::SettingsAiMessages,
    ) {
        SettingsOps::section_header(ui, &messages.capabilities_section);
        ui.add(crate::widgets::LabeledToggle::new(
            &messages.chat_enabled,
            &mut ollama.chat_enabled,
        ));
        ui.add(crate::widgets::LabeledToggle::new(
            &messages.autofix_enabled,
            &mut ollama.autofix_enabled,
        ));
    }

    fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                ui.label(label);
                ui.add(egui::TextEdit::singleline(value).desired_width(SETTINGS_TEXT_WIDTH));
            })
            .show(ui);
    }
}
