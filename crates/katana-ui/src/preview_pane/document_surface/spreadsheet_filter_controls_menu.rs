use eframe::egui;
use katana_document_viewer::SpreadsheetFilterCommand;

use super::{SpreadsheetFilterUiState, apply_command};

const CANDIDATE_SCROLL_HEIGHT: f32 = 240.0;
const ACTION_BUTTON_SPACING: f32 = 8.0;

pub(super) fn filter_menu(
    ui: &mut egui::Ui,
    messages: &crate::i18n::SpreadsheetFilterMessages,
    state: &mut SpreadsheetFilterUiState,
    sheet_index: usize,
    column: usize,
) -> Option<SpreadsheetFilterCommand> {
    if state.requested != Some((sheet_index, column)) {
        ui.label(&messages.title);
        return None;
    }
    ui.label(&messages.title);
    if state.loading {
        ui.label(&messages.loading);
        return None;
    }
    if state.candidate_truncated {
        ui.colored_label(ui.visuals().warn_fg_color, &messages.truncated);
    }
    if state.candidate_values.is_empty() {
        ui.label(&messages.no_values);
    } else {
        if ui.button(&messages.select_all).clicked() {
            state.selected_values = state.candidate_values.iter().cloned().collect();
        }
        show_candidate_values(ui, state);
    }
    show_blank_toggle(ui, state, &messages.blank);
    action_buttons(ui, messages, state, sheet_index, column)
}

fn show_candidate_values(ui: &mut egui::Ui, state: &mut SpreadsheetFilterUiState) {
    egui::ScrollArea::vertical()
        .max_height(CANDIDATE_SCROLL_HEIGHT)
        .show(ui, |ui| {
            for value in state.candidate_values.clone() {
                update_candidate_value(ui, state, value);
            }
        });
}

fn update_candidate_value(ui: &mut egui::Ui, state: &mut SpreadsheetFilterUiState, value: String) {
    let mut selected = state.selected_values.contains(&value);
    let was_selected = selected;
    crate::widgets::AlignCenter::new()
        .shrink_to_fit(true)
        .left(|ui| ui.label(&value))
        .right(|ui| crate::widgets::ToggleOps::switch(ui, &mut selected))
        .show(ui);
    if selected != was_selected {
        if selected {
            state.selected_values.insert(value);
        } else {
            state.selected_values.remove(&value);
        }
    }
}

fn show_blank_toggle(ui: &mut egui::Ui, state: &mut SpreadsheetFilterUiState, label: &str) {
    crate::widgets::AlignCenter::new()
        .shrink_to_fit(true)
        .left(|ui| ui.label(label))
        .right(|ui| crate::widgets::ToggleOps::switch(ui, &mut state.include_blank))
        .show(ui);
}

fn action_buttons(
    ui: &mut egui::Ui,
    messages: &crate::i18n::SpreadsheetFilterMessages,
    state: &mut SpreadsheetFilterUiState,
    sheet_index: usize,
    column: usize,
) -> Option<SpreadsheetFilterCommand> {
    let mut command = None;
    crate::widgets::AlignCenter::new()
        .shrink_to_fit(true)
        .content(|ui| {
            command = apply_button(ui, messages, state, sheet_index, column);
            ui.add_space(ACTION_BUTTON_SPACING);
            if clear_button(ui, messages, state) {
                command = Some(SpreadsheetFilterCommand::Clear {
                    sheet_index,
                    column: Some(column),
                });
            }
        })
        .show(ui);
    command
}

fn apply_button(
    ui: &mut egui::Ui,
    messages: &crate::i18n::SpreadsheetFilterMessages,
    state: &SpreadsheetFilterUiState,
    sheet_index: usize,
    column: usize,
) -> Option<SpreadsheetFilterCommand> {
    let enabled = !state.candidate_truncated && !state.has_unsupported_criterion;
    let response = ui.add_enabled(enabled, egui::Button::new(&messages.apply));
    if !response.clicked() {
        return None;
    }
    let command = apply_command(state, sheet_index, column);
    if command.is_some() {
        ui.close();
    }
    command
}

fn clear_button(
    ui: &mut egui::Ui,
    messages: &crate::i18n::SpreadsheetFilterMessages,
    state: &mut SpreadsheetFilterUiState,
) -> bool {
    if !ui.button(&messages.clear).clicked() {
        return false;
    }
    state.selected_values.clear();
    state.include_blank = false;
    state.include_non_blank = false;
    ui.close();
    true
}
