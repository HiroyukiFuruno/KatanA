use super::command_palette_results::execute_payload;
use crate::app_state::AppAction;
use crate::state::command_palette::CommandPaletteState;
use eframe::egui;

pub(super) fn handle_key_input(
    ui: &mut egui::Ui,
    state: &mut crate::state::command_palette::CommandPaletteState,
    action: &mut AppAction,
    is_open: &mut bool,
) -> bool {
    let mut changed_by_keyboard = false;
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        *is_open = false;
        return changed_by_keyboard;
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        state.move_down();
        changed_by_keyboard = true;
    }
    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        state.move_up();
        changed_by_keyboard = true;
    }
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let res = state.results.get(state.selected_index).cloned();
        if let Some(res) = res {
            execute_payload(action, &res.execute_payload);
            *is_open = false;
        }
    }
    changed_by_keyboard
}
