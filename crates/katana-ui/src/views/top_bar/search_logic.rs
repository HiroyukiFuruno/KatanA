use crate::app_state::AppAction;
use eframe::egui;

/* WHY: Width for the inline clear button 'x'. */
pub const DOC_SEARCH_CLEAR_BTN_WIDTH: f32 = 26.0;
/* WHY: Font size for the 'x' clear button. */
pub const DOC_SEARCH_CLEAR_BTN_FONT_SIZE: f32 = 14.0;
/* WHY: Alpha transparency for the clear button. */
pub const DOC_SEARCH_CLEAR_BTN_ALPHA: f32 = 0.5;

pub struct SearchLogic;

impl SearchLogic {
    pub fn render_clear_button(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        search_state: &mut crate::state::search::SearchState,
        action: &mut Option<AppAction>,
        response: &egui::Response,
    ) {
        /* WHY: Overlay inline clear button 'x' on the far right of the text input. */
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
                *action = Some(AppAction::DocSearchQueryChanged);
                response.request_focus();
            }
        });
    }

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
