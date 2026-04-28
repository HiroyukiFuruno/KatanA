use eframe::egui;
use katana_platform::DiffViewMode;

pub struct BehaviorDiffModeOps;

impl BehaviorDiffModeOps {
    pub(super) fn render(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let behavior_msgs = &crate::i18n::I18nOps::get().settings.behavior;
        let current = state.config.settings.settings().behavior.diff_view_mode;

        crate::widgets::AlignCenter::new()
            .left(|ui| ui.label(&behavior_msgs.diff_view_mode))
            .right(|ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        Self::render_mode_button(
                            ui,
                            state,
                            current,
                            DiffViewMode::Split,
                            &behavior_msgs.diff_view_mode_split,
                        );
                        Self::render_mode_button(
                            ui,
                            state,
                            current,
                            DiffViewMode::Inline,
                            &behavior_msgs.diff_view_mode_inline,
                        );
                    })
                    .show(ui)
            })
            .show(ui);
    }

    fn render_mode_button(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        current: DiffViewMode,
        target: DiffViewMode,
        label: &str,
    ) {
        let changed = ui
            .add(egui::Button::selectable(current == target, label).frame_when_inactive(true))
            .clicked()
            && current != target;
        if changed {
            state.config.settings.settings_mut().behavior.diff_view_mode = target;
            let _ = state.config.try_save_settings();
        }
    }
}
