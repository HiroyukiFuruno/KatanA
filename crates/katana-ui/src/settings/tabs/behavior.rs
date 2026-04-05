use super::types::*;
use crate::app_state::AppAction;
use crate::settings::*;

impl BehaviorTabOps {
    pub(crate) fn render_behavior_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) -> Option<AppAction> {
        let behavior_msgs = &crate::i18n::I18nOps::get().settings.behavior;

        let mut confirm = state
            .config
            .settings
            .settings()
            .behavior
            .confirm_close_dirty_tab;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &behavior_msgs.confirm_close_dirty_tab,
                    &mut confirm,
                )
                .position(crate::widgets::TogglePosition::Right)
                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            state
                .config
                .settings
                .settings_mut()
                .behavior
                .confirm_close_dirty_tab = confirm;
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SUBSECTION_SPACING);

        let mut scroll_sync = state
            .config
            .settings
            .settings()
            .behavior
            .scroll_sync_enabled;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(&behavior_msgs.scroll_sync, &mut scroll_sync)
                    .position(crate::widgets::TogglePosition::Right)
                    .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            state
                .config
                .settings
                .settings_mut()
                .behavior
                .scroll_sync_enabled = scroll_sync;
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SUBSECTION_SPACING);

        ui.add_space(SUBSECTION_SPACING);

        let mut enabled = state.config.settings.settings().behavior.auto_save;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(&behavior_msgs.auto_save, &mut enabled)
                    .position(crate::widgets::TogglePosition::Right)
                    .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            state.config.settings.settings_mut().behavior.auto_save = enabled;
            let _ = state.config.try_save_settings();
        }

        if enabled {
            ui.add_space(SETTINGS_TOGGLE_SPACING);

            let interval = state
                .config
                .settings
                .settings()
                .behavior
                .auto_save_interval_secs;
            ui.label(&behavior_msgs.auto_save_interval);

            let original_width = ui.spacing().slider_width;
            const SETTINGS_SLIDER_WIDTH: f32 = 300.0;
            ui.spacing_mut().slider_width = SETTINGS_SLIDER_WIDTH;

            // WHY: allow(horizontal_layout)
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    let mut display_val = interval;

                    let slider = egui::Slider::new(
                        &mut display_val,
                        AUTO_SAVE_INTERVAL_MIN..=AUTO_SAVE_INTERVAL_MAX,
                    )
                    .show_value(false) // WHY: Text is displayed separately
                    .step_by(AUTO_SAVE_INTERVAL_STEP)
                    .min_decimals(1)
                    .max_decimals(1)
                    .logarithmic(true)
                    .clamping(egui::SliderClamping::Always);

                    let slider_response = SettingsOps::add_styled_slider(ui, slider);

                    let drag_response = ui.add(
                        egui::DragValue::new(&mut display_val)
                            .speed(AUTO_SAVE_INTERVAL_STEP)
                            .suffix("s")
                            .max_decimals(1)
                            .range(AUTO_SAVE_INTERVAL_MIN..=AUTO_SAVE_INTERVAL_MAX),
                    );

                    if slider_response.changed() || drag_response.changed() {
                        state
                            .config
                            .settings
                            .settings_mut()
                            .behavior
                            .auto_save_interval_secs = display_val;
                        let _ = state.config.try_save_settings();
                    }
                })
                .show(ui);

            ui.spacing_mut().slider_width = original_width;
        }

        ui.add_space(SUBSECTION_SPACING);

        if ui.button(&behavior_msgs.clear_http_cache).clicked() {
            return Some(AppAction::ClearAllCaches);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_save_interval_slider_config_invariants() {
        assert_eq!(
            AUTO_SAVE_INTERVAL_STEP, 0.1,
            "The auto-save slider MUST increment/decrement by exactly 0.1 seconds \
             to satisfy the UI precision requirements."
        );
        assert_eq!(
            AUTO_SAVE_INTERVAL_MIN, 0.0,
            "The minimum auto-save interval MUST be 0.0 (off or immediate)."
        );
        assert_eq!(
            AUTO_SAVE_INTERVAL_MAX, 300.0,
            "The maximum auto-save interval MUST be 300.0 seconds."
        );
    }
}
