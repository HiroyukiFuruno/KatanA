use super::types::*;
use crate::app_state::AppAction;
use crate::settings::*;

mod editor;

impl BehaviorTabOps {
    pub(crate) fn render_behavior_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) -> Option<AppAction> {
        editor::BehaviorEditorOps::render(ui, state);
        ui.add_space(SUBSECTION_SPACING);

        let behavior_msgs = &crate::i18n::I18nOps::get().settings.behavior;
        let preview_msgs = &crate::i18n::I18nOps::get().preview;
        ui.label(egui::RichText::new(&preview_msgs.slideshow_settings).strong());
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let mut slideshow_hover = state
            .config
            .settings
            .settings()
            .behavior
            .slideshow_hover_highlight;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &preview_msgs.highlight_hover,
                    &mut slideshow_hover,
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
                .slideshow_hover_highlight = slideshow_hover;
            state.layout.slideshow_hover_highlight = slideshow_hover;
            let _ = state.config.try_save_settings();
        }
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let mut auto_hide = state
            .config
            .settings
            .settings()
            .behavior
            .slideshow_show_diagram_controls;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &preview_msgs.show_diagram_controls,
                    &mut auto_hide,
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
                .slideshow_show_diagram_controls = auto_hide;
            state.layout.slideshow_show_diagram_controls = auto_hide;
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SUBSECTION_SPACING);

        let mut toc_default = state.config.settings.settings().layout.toc_default_visible;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &behavior_msgs.toc_default_visible,
                    &mut toc_default,
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
                .layout
                .toc_default_visible = toc_default;
            let _ = state.config.try_save_settings();
        }
        ui.add_space(SETTINGS_TOGGLE_SPACING);

        let mut explorer_default = state
            .config
            .settings
            .settings()
            .layout
            .explorer_default_visible;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &behavior_msgs.explorer_default_visible,
                    &mut explorer_default,
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
                .layout
                .explorer_default_visible = explorer_default;
            let _ = state.config.try_save_settings();
        }

        ui.add_space(SUBSECTION_SPACING);

        if ui.button(&behavior_msgs.clear_http_cache).clicked() {
            return Some(AppAction::ClearAllCaches);
        }

        None
    }
}
