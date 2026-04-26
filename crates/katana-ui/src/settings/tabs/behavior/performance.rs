use crate::app_state::AppState;
use crate::settings::*;
use eframe::egui;

pub struct BehaviorPerformanceOps;

impl BehaviorPerformanceOps {
    pub(super) fn render(ui: &mut egui::Ui, state: &mut AppState) {
        let msgs = &crate::i18n::I18nOps::get().settings.behavior;

        ui.label(egui::RichText::new(&msgs.diagram_rendering_section_title).strong());
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let is_unlimited = state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency_unlimited;
        let mut concurrency = state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        ui.add_enabled_ui(!is_unlimited, |ui| {
            crate::widgets::AlignCenter::new()
                .content(|ui| {
                    ui.label(&msgs.diagram_concurrency);
                    if ui
                        .add(egui::DragValue::new(&mut concurrency).speed(1.0))
                        .changed()
                    {
                        state
                            .config
                            .settings
                            .settings_mut()
                            .performance
                            .diagram_concurrency = concurrency.max(1);
                        let _ = state.config.try_save_settings();
                    }
                })
                .show(ui);
        });

        let mut unlimited = is_unlimited;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &msgs.diagram_concurrency_unlimited,
                    &mut unlimited,
                )
                .position(crate::widgets::TogglePosition::Right)
                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            if unlimited {
                Self::open_unlimited_warning(ui);
            } else {
                Self::set_unlimited(state, false);
            }
        }

        ui.label(
            egui::RichText::new(&msgs.diagram_concurrency_hint)
                .size(HINT_FONT_SIZE)
                .color(ui.visuals().weak_text_color()),
        );

        Self::render_unlimited_warning(ui, state);
    }

    fn open_unlimited_warning(ui: &mut egui::Ui) {
        ui.data_mut(|it| it.insert_temp(Self::unlimited_modal_id(), true));
    }

    fn set_unlimited(state: &mut AppState, enabled: bool) {
        state
            .config
            .settings
            .settings_mut()
            .performance
            .diagram_concurrency_unlimited = enabled;
        let _ = state.config.try_save_settings();
    }

    fn render_unlimited_warning(ui: &mut egui::Ui, state: &mut AppState) {
        if !ui.data(|it| {
            it.get_temp::<bool>(Self::unlimited_modal_id())
                .unwrap_or(false)
        }) {
            return;
        }

        let msgs = &crate::i18n::I18nOps::get().settings.behavior;
        let mut close = false;
        let mut confirm = false;
        egui::Window::new(&msgs.diagram_concurrency_unlimited_warning_title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label(&msgs.diagram_concurrency_unlimited_warning_message);
                ui.add_space(SUBSECTION_SPACING);
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        if ui
                            .button(crate::i18n::I18nOps::get().action.cancel.clone())
                            .clicked()
                        {
                            close = true;
                        }
                        if ui
                            .button(crate::i18n::I18nOps::get().action.confirm.clone())
                            .clicked()
                        {
                            confirm = true;
                            close = true;
                        }
                    })
                    .show(ui);
            });

        if confirm {
            Self::set_unlimited(state, true);
        }
        if close || ui.input(|it| it.key_pressed(egui::Key::Escape)) {
            ui.data_mut(|it| it.insert_temp(Self::unlimited_modal_id(), false));
        }
    }

    fn unlimited_modal_id() -> egui::Id {
        egui::Id::new("diagram_concurrency_unlimited_warning")
    }
}
