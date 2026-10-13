use super::types::*;
use crate::settings::*;

impl LayoutTabOps {
    pub(crate) fn render_layout_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let title = crate::i18n::I18nOps::get().settings.tab_name("layout");
        SettingsOps::section_header(ui, &title);
        Self::render_toc_toggle(ui, state);
        ui.add_space(SECTION_SPACING);
        Self::render_toc_position_selector(ui, state);
        ui.add_space(SECTION_SPACING);
        Self::render_split_direction_selector(ui, state);
        ui.add_space(LAYOUT_SELECTOR_SPACING);
        Self::render_pane_order_selector(ui, state);
        ui.add_space(SECTION_SPACING);
        Self::render_accordion_vertical_line_toggle(ui, state);
    }

    pub(crate) fn render_accordion_vertical_line_toggle(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let mut enabled = state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &crate::i18n::I18nOps::get()
                        .settings
                        .layout
                        .accordion_vertical_line,
                    &mut enabled,
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
                .accordion_vertical_line = enabled;
            let _ = state.config.try_save_settings();
        }
    }

    pub(crate) fn render_toc_toggle(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        let mut toc_visible = state.config.settings.settings().layout.toc_visible;
        if ui
            .add(
                crate::widgets::LabeledToggle::new(
                    &crate::i18n::I18nOps::get().settings.toc_visible,
                    &mut toc_visible,
                )
                .position(crate::widgets::TogglePosition::Right)
                .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
            )
            .changed()
        {
            state.config.settings.settings_mut().layout.toc_visible = toc_visible;
            let _ = state.config.try_save_settings();
        }
    }

    pub(crate) fn render_toc_position_selector(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        if !state.config.settings.settings().layout.toc_visible {
            return;
        }

        use katana_platform::settings::TocPosition;

        ui.label(
            crate::i18n::I18nOps::get()
                .settings
                .layout
                .toc_position
                .clone(),
        );
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                let current = state.config.settings.settings().layout.toc_position;
                if ui
                    /* WHY: in popup/list context; future: standardize as atom */
                    .add(
                        egui::Button::selectable(
                            current == TocPosition::Left,
                            crate::i18n::I18nOps::get().settings.layout.left.clone(),
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                    && current != TocPosition::Left
                {
                    state.config.settings.settings_mut().layout.toc_position = TocPosition::Left;
                    let _ = state.config.try_save_settings();
                }
                if ui
                    /* WHY: in popup/list context; future: standardize as atom */
                    .add(
                        egui::Button::selectable(
                            current == TocPosition::Right,
                            crate::i18n::I18nOps::get().settings.layout.right.clone(),
                        )
                        .frame_when_inactive(true),
                    )
                    .clicked()
                    && current != TocPosition::Right
                {
                    state.config.settings.settings_mut().layout.toc_position = TocPosition::Right;
                    let _ = state.config.try_save_settings();
                }
            })
            .show(ui);
    }

    pub(crate) fn render_string_list_editor(ui: &mut egui::Ui, list: &mut Vec<String>) -> bool {
        let mut changed = false;
        let mut to_remove = None;

        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
            for (i, item) in list.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    crate::widgets::AlignCenter::new()
                        .shrink_to_fit(true)
                        .left(|ui| {
                            const STRING_LIST_INPUT_WIDTH: f32 = 140.0;
                            let response = ui.add(
                                egui::TextEdit::singleline(item)
                                    .desired_width(STRING_LIST_INPUT_WIDTH),
                            );
                            if response.changed() {
                                changed = true;
                            }
                            ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                        })
                        .right(|ui| {
                            if ui.button("-").clicked() {
                                to_remove = Some(i);
                            }
                            ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                        })
                        .show(ui);
                });
            }

            if let Some(i) = to_remove {
                list.remove(i);
                changed = true;
            }

            if ui.button("+").clicked() {
                list.push(String::new());
                changed = true;
            }
        });

        changed
    }
}
