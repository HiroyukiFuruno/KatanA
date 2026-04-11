use super::search_logic::SearchLogic;
use crate::app_state::AppAction;
use eframe::egui;

/* WHY: Vertical adjustment for the search bar height calculation. */
const SEARCH_BAR_HEIGHT_ADJUSTMENT: f32 = 10.0;
/* WHY: Preferred width for the document search input field. */
const DOC_SEARCH_INPUT_WIDTH: f32 = 200.0;
/* WHY: Horizontal margin for the search input content (icons). */
const DOC_SEARCH_INPUT_MARGIN_X: f32 = 26.0;
/* WHY: Vertical margin for the search input content. */
const DOC_SEARCH_INPUT_MARGIN_Y: f32 = 4.0;
/* WHY: Fixed width for the overlay icons inside the search input. */
const SEARCH_ICON_WIDTH: f32 = 26.0;
/* WHY: Alpha transparency for the search icon when dimmed. */
const SEARCH_ICON_DIM_ALPHA: f32 = 0.5;

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

        Self::render_nav_buttons(ui, action, button_size);
        Self::render_match_count(ui, search_state);

        let response = ui.add(
            egui::TextEdit::singleline(&mut search_state.doc_search_query)
                .desired_width(DOC_SEARCH_INPUT_WIDTH)
                .margin(egui::Margin::symmetric(
                    DOC_SEARCH_INPUT_MARGIN_X as i8,
                    DOC_SEARCH_INPUT_MARGIN_Y as i8,
                ))
                .id_source("doc_search_input_stable_id"),
        );

        let rect = response.rect;
        Self::render_input_overlay(ui, rect);

        if !search_state.doc_search_query.is_empty() {
            SearchLogic::render_clear_button(ui, rect, search_state, action, &response);
        }

        SearchLogic::handle_input_events(ui, &response, action, search_state);
    }

    fn render_nav_buttons(
        ui: &mut egui::Ui,
        action: &mut Option<AppAction>,
        button_size: egui::Vec2,
    ) {
        if ui
            .add(
                crate::Icon::PanDown
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
                crate::Icon::PanUp
                    .button(ui, crate::icon::IconSize::Medium)
                    .min_size(button_size),
            )
            .on_hover_text(crate::i18n::I18nOps::get().search.doc_search_prev.clone())
            .clicked()
        {
            *action = Some(AppAction::DocSearchPrev);
        }
    }

    fn render_match_count(ui: &mut egui::Ui, search_state: &crate::state::search::SearchState) {
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
    }

    fn render_input_overlay(ui: &mut egui::Ui, rect: egui::Rect) {
        let left_icon_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(SEARCH_ICON_WIDTH, rect.height()));
        ui.allocate_ui_at_rect(left_icon_rect, |ui| {
            ui.centered_and_justified(|ui| {
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
    }
}
