use crate::app_state::AppAction;
use eframe::egui;

pub struct SearchLogic;

impl SearchLogic {
    pub fn handle_input_events(
        ui: &mut egui::Ui,
        response: &egui::Response,
        action: &mut Option<AppAction>,
        search_state: &mut crate::state::search::SearchState,
    ) {
        let is_newly_opened = ui.memory(|m| {
            m.data
                .get_temp::<bool>(egui::Id::new("search_newly_opened"))
                .unwrap_or(false)
        });

        if is_newly_opened {
            response.request_focus();
            ui.memory_mut(|m| {
                m.data
                    .insert_temp(egui::Id::new("search_newly_opened"), false)
            });
        }

        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let shift_pressed = ui.input(|i| i.modifiers.shift);
        let up_pressed = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let down_pressed =
            response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown));

        if (enter_pressed && !shift_pressed) || down_pressed {
            *action = Some(AppAction::DocSearchNext);
        } else if (enter_pressed && shift_pressed) || up_pressed {
            *action = Some(AppAction::DocSearchPrev);
        }

        if response.changed() {
            *action = Some(AppAction::DocSearchQueryChanged);
        }

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            search_state.doc_search_open = false;
        }
    }
}
