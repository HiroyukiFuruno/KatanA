use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct DocSearchBar;

impl DocSearchBar {
    pub fn show(
        ui: &mut egui::Ui,
        search_state: &mut crate::state::search::SearchState,
    ) -> Option<AppAction> {
        let mut action = None;
        let available_width = ui.available_width();
        let bar_height = ui.spacing().interact_size.y;

        ui.separator();
        ui.allocate_ui_with_layout(
            {
                const SEARCH_BAR_HEIGHT_ADJUSTMENT: f32 = 10.0;
                egui::vec2(available_width, bar_height + SEARCH_BAR_HEIGHT_ADJUSTMENT)
            },
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let button_size =
                    egui::vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);
                /* WHY: Drawing right-to-left, so we add: Close, Next, Prev, MatchCount, Input */
                if ui
                    .add(
                        crate::Icon::Close
                            .button(ui, crate::icon::IconSize::Medium)
                            .min_size(button_size),
                    )
                    .on_hover_text(crate::i18n::I18nOps::get().search.doc_search_close.clone())
                    .clicked()
                {
                    search_state.doc_search_open = false;
                }

                if ui
                    .add(
                        crate::Icon::PanDown
                            .button(ui, crate::icon::IconSize::Medium)
                            .min_size(button_size),
                    )
                    .on_hover_text(crate::i18n::I18nOps::get().search.doc_search_next.clone())
                    .clicked()
                {
                    action = Some(AppAction::DocSearchNext);
                }

                if ui
                    .add(
                        crate::Icon::PanUp
                            .button(ui, crate::icon::IconSize::Medium)
                            .min_size(button_size),
                    )
                    .on_hover_text(crate::i18n::I18nOps::get().search.doc_search_prev.clone())
                    .clicked()
                {
                    action = Some(AppAction::DocSearchPrev);
                }

                let match_count = search_state.doc_search_matches.len();
                if match_count > 0 {
                    ui.label(crate::i18n::I18nOps::tf(
                        &crate::i18n::I18nOps::get().search.doc_search_count,
                        &[
                            (
                                "index",
                                &format!("{}", search_state.doc_search_active_index + 1),
                            ),
                            ("total", &format!("{}", match_count)),
                        ],
                    ));
                } else if !search_state.doc_search_query.is_empty() {
                    ui.label(crate::i18n::I18nOps::tf(
                        &crate::i18n::I18nOps::get().search.doc_search_count,
                        &[("index", "0"), ("total", "0")],
                    ));
                }

                const DOC_SEARCH_INPUT_WIDTH: f32 = 200.0;
                let is_newly_opened = ui.memory(|m| {
                    m.data
                        .get_temp::<bool>(egui::Id::new("search_newly_opened"))
                        .unwrap_or(false)
                });

                /* WHY: Add text edit with margin to fit icons on both sides */
                const DOC_SEARCH_INPUT_MARGIN_X: i8 = 26;
                const DOC_SEARCH_INPUT_MARGIN_Y: i8 = 4;
                let response = ui.add(
                    egui::TextEdit::singleline(&mut search_state.doc_search_query)
                        .desired_width(DOC_SEARCH_INPUT_WIDTH)
                        .margin(egui::Margin {
                            left: DOC_SEARCH_INPUT_MARGIN_X,
                            right: DOC_SEARCH_INPUT_MARGIN_X,
                            top: DOC_SEARCH_INPUT_MARGIN_Y,
                            bottom: DOC_SEARCH_INPUT_MARGIN_Y,
                        })
                        .vertical_align(egui::Align::Center)
                        .id_source("doc_search_input_stable_id"),
                );

                let rect = response.rect;

                /* WHY: Overlay readonly search icon on the left */
                const SEARCH_ICON_WIDTH: f32 = 26.0;
                let left_icon_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.min.y),
                    egui::vec2(SEARCH_ICON_WIDTH, rect.height()),
                );
                ui.allocate_ui_at_rect(left_icon_rect, |ui| {
                    ui.centered_and_justified(|ui| {
                        const SEARCH_ICON_DIM_ALPHA: f32 = 0.5;
                        let icon_color = ui
                            .visuals()
                            .text_color()
                            .gamma_multiply(SEARCH_ICON_DIM_ALPHA);
                        ui.add(
                            crate::Icon::Search
                                .image(crate::icon::IconSize::Small)
                                .tint(icon_color),
                        );
                    });
                });

                /* WHY: Overlay inline clear button 'x' on the right side */
                if !search_state.doc_search_query.is_empty() {
                    const DOC_SEARCH_CLEAR_BTN_WIDTH: f32 = 26.0;
                    const DOC_SEARCH_CLEAR_BTN_FONT_SIZE: f32 = 14.0;
                    const DOC_SEARCH_CLEAR_BTN_ALPHA: f32 = 0.5;
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.max.x - DOC_SEARCH_CLEAR_BTN_WIDTH, rect.min.y),
                        egui::vec2(DOC_SEARCH_CLEAR_BTN_WIDTH, rect.height()),
                    );
                    ui.allocate_ui_at_rect(btn_rect, |ui| {
                        let btn = egui::Button::new(
                            egui::RichText::new("×")
                                .size(DOC_SEARCH_CLEAR_BTN_FONT_SIZE)
                                .color(
                                    ui.visuals()
                                        .text_color()
                                        .gamma_multiply(DOC_SEARCH_CLEAR_BTN_ALPHA),
                                ),
                        )
                        .frame(false);

                        if ui.centered_and_justified(|ui| ui.add(btn)).inner.clicked() {
                            search_state.doc_search_query.clear();
                            action = Some(AppAction::DocSearchQueryChanged);
                            response.request_focus();
                        }
                    });
                }

                if is_newly_opened {
                    response.request_focus();
                    ui.memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new("search_newly_opened"), false)
                    });
                }

                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let shift_pressed = ui.input(|i| i.modifiers.shift);
                let up_pressed =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                let down_pressed =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown));

                if (enter_pressed && !shift_pressed) || down_pressed {
                    action = Some(AppAction::DocSearchNext);
                } else if (enter_pressed && shift_pressed) || up_pressed {
                    action = Some(AppAction::DocSearchPrev);
                }

                if response.changed() {
                    action = Some(AppAction::DocSearchQueryChanged);
                }

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    search_state.doc_search_open = false;
                }
            },
        );

        action
    }
}
