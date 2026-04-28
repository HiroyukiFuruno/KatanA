use eframe::egui;

const MIN_TERM_WIDTH: f32 = 10.0;

pub(super) enum SearchHistoryAction {
    Select(String),
    Remove(String),
}

pub(super) struct SearchHistoryUiOps;

impl SearchHistoryUiOps {
    pub(super) fn apply_keyboard_navigation(
        ui: &egui::Ui,
        response: &egui::Response,
        search: &mut crate::app_state::SearchState,
    ) -> bool {
        if !response.has_focus() || search.md_history.recent_terms.is_empty() {
            return false;
        }

        if ui.input(|it| it.key_pressed(egui::Key::ArrowUp)) {
            Self::restore_older(search);
            return true;
        }
        if ui.input(|it| it.key_pressed(egui::Key::ArrowDown)) {
            Self::restore_newer(search);
            return true;
        }
        false
    }

    pub(super) fn reset_navigation(search: &mut crate::app_state::SearchState) {
        search.md_history_cursor = None;
        search.md_history_draft.clear();
    }

    pub(super) fn render_recent_terms(
        ui: &mut egui::Ui,
        terms: &[String],
    ) -> Vec<SearchHistoryAction> {
        let mut actions = Vec::new();
        for term in terms {
            if let Some(action) = Self::render_recent_term(ui, term) {
                actions.push(action);
            }
        }
        actions
    }

    fn restore_older(search: &mut crate::app_state::SearchState) {
        let next_index = match search.md_history_cursor {
            Some(index) => (index + 1).min(search.md_history.recent_terms.len() - 1),
            None => {
                search.md_history_draft = search.md_search.query.clone();
                0
            }
        };
        search.md_history_cursor = Some(next_index);
        search.md_search.query = search.md_history.recent_terms[next_index].clone();
    }

    fn restore_newer(search: &mut crate::app_state::SearchState) {
        let Some(index) = search.md_history_cursor else {
            return;
        };
        if index == 0 {
            search.md_search.query = search.md_history_draft.clone();
            Self::reset_navigation(search);
            return;
        }

        let next_index = index - 1;
        search.md_history_cursor = Some(next_index);
        search.md_search.query = search.md_history.recent_terms[next_index].clone();
    }

    fn render_recent_term(ui: &mut egui::Ui, term: &str) -> Option<SearchHistoryAction> {
        let mut action = None;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let remove_width =
                    crate::icon::IconSize::Small.to_vec2().x + ui.spacing().button_padding.x * 2.0;
                let term_width =
                    (ui.available_width() - remove_width - ui.spacing().item_spacing.x)
                        .max(MIN_TERM_WIDTH);

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_min_width(term_width);
                    ui.set_max_width(term_width);
                    if ui
                        .add(egui::Button::new(term).frame(false).truncate())
                        .clicked()
                    {
                        action = Some(SearchHistoryAction::Select(term.to_string()));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(crate::Icon::Remove.button(ui, crate::icon::IconSize::Small))
                        .on_hover_text(crate::i18n::I18nOps::get().search.clear_history.clone())
                        .clicked()
                    {
                        action = Some(SearchHistoryAction::Remove(term.to_string()));
                    }
                });
            },
        );
        action
    }
}
