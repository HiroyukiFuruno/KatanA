use eframe::egui;
use katana_platform::settings::PresetReference;
use katana_platform::settings::PresetState;

pub(crate) struct PresetWidgetLabels<'a> {
    pub title: &'a str,
    pub save: &'a str,
    pub revert: &'a str,
    pub advanced: Option<&'a str>,
}

#[derive(Default)]
pub(crate) struct PresetWidgetResponse {
    pub selected: Option<PresetReference>,
    pub save_clicked: bool,
    pub revert_clicked: bool,
    pub advanced_clicked: bool,
}

pub(crate) struct PresetWidgetOps;

impl PresetWidgetOps {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        id_source: &'static str,
        state: &PresetState,
        built_in_presets: &[PresetReference],
        user_presets: &[PresetReference],
        labels: PresetWidgetLabels<'_>,
    ) -> PresetWidgetResponse {
        crate::settings::SettingsOps::section_header(ui, labels.title);
        let mut response = PresetWidgetResponse::default();

        crate::widgets::AlignCenter::new()
            .content(|ui| {
                Self::render_combo(
                    ui,
                    id_source,
                    state,
                    built_in_presets,
                    user_presets,
                    &mut response,
                );
                Self::render_actions(ui, state, labels, &mut response);
            })
            .show(ui);

        response
    }

    fn render_combo(
        ui: &mut egui::Ui,
        id_source: &'static str,
        state: &PresetState,
        built_in_presets: &[PresetReference],
        user_presets: &[PresetReference],
        response: &mut PresetWidgetResponse,
    ) {
        egui::ComboBox::from_id_source(id_source)
            .selected_text(Self::selected_text(state))
            .show_ui(ui, |ui| {
                Self::render_options(ui, state, built_in_presets, response);
                if !user_presets.is_empty() {
                    ui.separator();
                    Self::render_options(ui, state, user_presets, response);
                }
            });
    }

    fn render_options(
        ui: &mut egui::Ui,
        state: &PresetState,
        presets: &[PresetReference],
        response: &mut PresetWidgetResponse,
    ) {
        for preset in presets {
            let selected = state
                .current
                .as_ref()
                .is_some_and(|current| current.source == preset.source && current.id == preset.id);
            if ui
                .add(egui::Button::selectable(selected, &preset.label).frame_when_inactive(true))
                .clicked()
            {
                response.selected = Some(preset.clone());
            }
        }
    }

    fn render_actions(
        ui: &mut egui::Ui,
        state: &PresetState,
        labels: PresetWidgetLabels<'_>,
        response: &mut PresetWidgetResponse,
    ) {
        if ui.button(labels.save).clicked() {
            response.save_clicked = true;
        }
        if ui
            .add_enabled(
                state.modified && state.base.is_some(),
                egui::Button::new(labels.revert),
            )
            .clicked()
        {
            response.revert_clicked = true;
        }
        if let Some(advanced) = labels.advanced
            && ui.button(advanced).clicked()
        {
            response.advanced_clicked = true;
        }
    }

    fn selected_text(state: &PresetState) -> String {
        let Some(current) = state.current.as_ref() else {
            return String::new();
        };
        if state.modified {
            format!("{} *", current.label)
        } else {
            current.label.clone()
        }
    }
}
