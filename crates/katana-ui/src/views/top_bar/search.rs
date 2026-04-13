use super::search_logic::SearchLogic;
use crate::app_state::AppAction;
use eframe::egui;

/* WHY: Vertical adjustment for the search bar height calculation. */
const SEARCH_BAR_HEIGHT_ADJUSTMENT: f32 = 10.0;
/* WHY: Preferred width for the document search input field. */
const DOC_SEARCH_INPUT_WIDTH: f32 = 200.0;

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
            egui::vec2(available_width, bar_height + SEARCH_BAR_HEIGHT_ADJUSTMENT),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                Self::render_content(ui, search_state, &mut action);
            },
        );

        action
    }

    fn render_content(
        ui: &mut egui::Ui,
        search_state: &mut crate::state::search::SearchState,
        action: &mut Option<AppAction>,
    ) {
        let bar_height = ui.spacing().interact_size.y;
        let button_size = egui::vec2(bar_height, bar_height);

        /* WHY: Close button. */
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

        let has_matches = !search_state.doc_search_matches.is_empty();
        Self::render_nav_buttons(ui, action, button_size, has_matches);
        Self::render_match_count(ui, search_state);

        /* WHY: Use explicit id_source to prevent collision with other active search bars */
        let response = crate::widgets::SearchBar::new(&mut search_state.doc_search)
            .desired_width(DOC_SEARCH_INPUT_WIDTH)
            .hint_text(crate::i18n::I18nOps::get().search.doc_query_hint.clone())
            .id_source("doc_search_bar")
            .show(ui);

        SearchLogic::handle_input_events(ui, &response, action, search_state);
    }

    fn render_nav_buttons(
        ui: &mut egui::Ui,
        action: &mut Option<AppAction>,
        button_size: egui::Vec2,
        has_matches: bool,
    ) {
        ui.add_enabled_ui(has_matches, |ui| {
            if ui
                .add(
                    crate::Icon::ArrowDown
                        .button(ui, crate::icon::IconSize::Medium)
                        .min_size(button_size),
                )
                .on_hover_text(crate::i18n::I18nOps::get().search.doc_search_next.clone())
                .clicked()
            {
                *action = Some(AppAction::DocSearchNext);
            }

            if ui
                .add(
                    crate::Icon::ArrowUp
                        .button(ui, crate::icon::IconSize::Medium)
                        .min_size(button_size),
                )
                .on_hover_text(crate::i18n::I18nOps::get().search.doc_search_prev.clone())
                .clicked()
            {
                *action = Some(AppAction::DocSearchPrev);
            }
        });
    }

    fn render_match_count(ui: &mut egui::Ui, search_state: &crate::state::search::SearchState) {
        let match_count = search_state.doc_search_matches.len();

        const DOC_SEARCH_COUNT_WIDTH: f32 = 150.0;

        ui.allocate_ui_with_layout(
            egui::vec2(DOC_SEARCH_COUNT_WIDTH, ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_min_width(DOC_SEARCH_COUNT_WIDTH);
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
                } else {
                    let mut text = egui::RichText::new(
                        crate::i18n::I18nOps::get()
                            .search
                            .doc_search_no_results
                            .clone(),
                    );
                    if !search_state.doc_search.query.is_empty() {
                        text = text.color(ui.visuals().error_fg_color);
                    }
                    ui.label(text);
                }
            },
        );
    }
}
