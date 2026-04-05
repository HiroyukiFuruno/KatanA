use super::types::*;
use crate::settings::*;
use katana_platform::settings::{MAX_FONT_SIZE, MIN_FONT_SIZE};

impl FontTabOps {
    pub(crate) fn render_font_tab(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
        SettingsOps::section_header(ui, &crate::i18n::I18nOps::get().settings.font.size);
        Self::render_font_size_slider(ui, state);
        ui.add_space(SECTION_SPACING);
        SettingsOps::section_header(ui, &crate::i18n::I18nOps::get().settings.font.family);
        Self::render_font_family_selector(ui, state);
    }

    pub(crate) fn render_font_family_selector(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let current = state.config.settings.settings().font.family.clone();
        let os_fonts = katana_platform::os_fonts::OsFontScanner::cached_fonts();

        let open_id = egui::Id::new("font_selector_open");
        let search_id = egui::Id::new("font_search_query");

        let is_open: bool = ui.data(|d| d.get_temp(open_id)).unwrap_or(false);
        let mut query: String = ui
            .data(|d| d.get_temp::<String>(search_id))
            .unwrap_or_default();

        let button_resp = ui
            .add(egui::Button::new(&current).min_size(egui::vec2(FONT_FAMILY_COMBOBOX_WIDTH, 0.0)));
        if button_resp.clicked() {
            let new_state = !is_open;
            ui.data_mut(|d| d.insert_temp(open_id, new_state));
            if new_state {
                ui.data_mut(|d| d.insert_temp(search_id, String::new()));
                query = String::new();
            }
        }

        egui::Popup::from_response(&button_resp)
            .open(is_open)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_width(FONT_FAMILY_COMBOBOX_WIDTH);

                let search_resp = ui.text_edit_singleline(&mut query);
                if search_resp.changed() {
                    ui.data_mut(|d: &mut egui::util::IdTypeMap| {
                        d.insert_temp(search_id, query.clone())
                    });
                }
                search_resp.request_focus();

                ui.separator();

                let query_lower = query.to_lowercase();
                let mut selected: Option<String> = None;
                let mut close = false;

                egui::ScrollArea::vertical()
                    .max_height(FONT_DROPDOWN_MAX_HEIGHT)
                    .show(ui, |ui| {
                        let defaults = vec!["Proportional", "Monospace"];
                        for family in defaults {
                            if query_lower.is_empty()
                                || family.to_lowercase().contains(&query_lower)
                            {
                                let is_current = current == family;
                                /* WHY: in popup/list context; future: standardize as atom */
                                if ui
                                    .add(
                                        egui::Button::selectable(is_current, family)
                                            .frame_when_inactive(true),
                                    )
                                    .clicked()
                                {
                                    selected = Some(family.to_string());
                                    close = true;
                                }
                            }
                        }
                        ui.separator();
                        for (name, _) in os_fonts.iter() {
                            let name: &String = name;
                            if query_lower.is_empty() || name.to_lowercase().contains(&query_lower)
                            {
                                let is_current = current == *name;
                                /* WHY: in popup/list context; future: standardize as atom */
                                if ui
                                    .add(
                                        egui::Button::selectable(is_current, name.as_str())
                                            .frame_when_inactive(true),
                                    )
                                    .clicked()
                                {
                                    selected = Some(name.clone());
                                    close = true;
                                }
                            }
                        }
                    });

                if let Some(new_font) = selected {
                    state.config.settings.settings_mut().font.family = new_font;
                    let _ = state.config.try_save_settings();
                }
                let should_close = close || ui.input(|i| i.key_pressed(egui::Key::Escape));
                if should_close {
                    ui.data_mut(|d: &mut egui::util::IdTypeMap| {
                        d.insert_temp(open_id, false);
                        d.insert_temp(search_id, String::new());
                    });
                }
            });
    }

    pub(crate) fn render_font_size_slider(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
    ) {
        let mut size = state.config.settings.settings().clamped_font_size();
        let slider = egui::Slider::new(&mut size, MIN_FONT_SIZE..=MAX_FONT_SIZE)
            .step_by(FONT_SIZE_STEP)
            .suffix(" px");

        if SettingsOps::add_styled_slider(ui, slider)
            .on_hover_text(
                crate::i18n::I18nOps::get()
                    .settings
                    .font
                    .size_slider_hint
                    .clone(),
            )
            .changed()
        {
            state.config.settings.settings_mut().set_font_size(size);
            let _ = state.config.try_save_settings();
        }
    }
}
