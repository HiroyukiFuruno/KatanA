use crate::app_state::AppState;
use crate::settings::*;
use eframe::egui;

pub struct BehaviorIngestOps;

const INGEST_TEXT_FIELD_WIDTH: f32 = 300.0;

impl BehaviorIngestOps {
    pub(super) fn render(ui: &mut egui::Ui, state: &mut AppState) {
        let msgs = &crate::i18n::I18nOps::get().settings.behavior;

        ui.label(egui::RichText::new(&msgs.ingest_section_title).strong());
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        ui.indent("ingest_settings_body", |ui| {
            Self::render_body(ui, state, msgs);
        });
    }

    fn render_body(
        ui: &mut egui::Ui,
        state: &mut AppState,
        msgs: &crate::i18n::SettingsBehaviorMessages,
    ) {
        let mut save_dir = state
            .config
            .settings
            .settings()
            .ingest
            .image_save_directory
            .clone();
        crate::widgets::AlignCenter::new()
            .left(|ui| ui.label(&msgs.ingest_image_save_directory))
            .right(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut save_dir)
                        .desired_width(INGEST_TEXT_FIELD_WIDTH),
                );
                if response.changed() {
                    state
                        .config
                        .settings
                        .settings_mut()
                        .ingest
                        .image_save_directory = save_dir;
                    let _ = state.config.try_save_settings();
                }
                response
            })
            .show(ui);
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let mut name_format = state
            .config
            .settings
            .settings()
            .ingest
            .image_name_format
            .clone();
        crate::widgets::AlignCenter::new()
            .left(|ui| ui.label(&msgs.ingest_image_name_format))
            .right(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut name_format)
                        .desired_width(INGEST_TEXT_FIELD_WIDTH),
                );
                if response.changed() {
                    state
                        .config
                        .settings
                        .settings_mut()
                        .ingest
                        .image_name_format = name_format;
                    let _ = state.config.try_save_settings();
                }
                response
            })
            .show(ui);
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let mut create_dir = state
            .config
            .settings
            .settings()
            .ingest
            .create_directory_if_not_exists;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(&msgs.ingest_create_directory, &mut create_dir)
                    .position(crate::widgets::TogglePosition::Right)
                    .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            state
                .config
                .settings
                .settings_mut()
                .ingest
                .create_directory_if_not_exists = create_dir;
            let _ = state.config.try_save_settings();
        }
    }
}
