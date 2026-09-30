use std::collections::BTreeSet;

#[cfg(test)]
#[path = "spreadsheet_filter_controls_tests.rs"]
mod tests;

use eframe::egui;
use katana_document_viewer::{
    DocumentGridCell, DocumentGridSurfaceFrame, SpreadsheetFilterCommand,
    SpreadsheetFilterCriterion, SpreadsheetFilterEvent, SpreadsheetFrameMetadata,
};

const CANDIDATE_LIMIT: usize = 512;
const FILTER_BUTTON_WIDTH: f32 = 22.0;
const CANDIDATE_SCROLL_HEIGHT: f32 = 240.0;
const ACTION_BUTTON_SPACING: f32 = 8.0;

#[derive(Debug, Default)]
pub(super) struct SpreadsheetFilterUiState {
    pub(super) metadata: Option<SpreadsheetFrameMetadata>,
    pub(super) requested: Option<(usize, usize)>,
    pub(super) candidate_values: Vec<String>,
    pub(super) selected_values: BTreeSet<String>,
    pub(super) include_blank: bool,
    pub(super) include_non_blank: bool,
    pub(super) loading: bool,
    pub(super) candidate_truncated: bool,
    has_existing_filter: bool,
}

impl SpreadsheetFilterUiState {
    pub(super) fn update_metadata(
        &mut self,
        metadata: Option<SpreadsheetFrameMetadata>,
        event: Option<&SpreadsheetFilterEvent>,
    ) {
        let sheet_changed = self.metadata.as_ref().map(|current| current.sheet_index)
            != metadata.as_ref().map(|current| current.sheet_index);
        self.metadata = metadata;

        if sheet_changed {
            self.reset_request();
        }

        match event {
            Some(SpreadsheetFilterEvent::Candidates {
                sheet_index,
                column,
                values,
                truncated,
            }) if self.requested == Some((*sheet_index, *column))
                && self
                    .metadata
                    .as_ref()
                    .is_some_and(|current| current.sheet_index == *sheet_index) =>
            {
                let contains_blank = values.iter().any(String::is_empty);
                self.candidate_values = values
                    .iter()
                    .filter(|value| !value.is_empty())
                    .cloned()
                    .collect();
                self.candidate_truncated = *truncated;
                if !self.has_existing_filter {
                    self.selected_values = self.candidate_values.iter().cloned().collect();
                    self.include_blank = contains_blank;
                } else if self.include_non_blank {
                    self.selected_values
                        .extend(self.candidate_values.iter().cloned());
                }
                self.loading = false;
            }
            Some(SpreadsheetFilterEvent::VisibilityChanged { sheet_index, .. })
                if self
                    .metadata
                    .as_ref()
                    .is_some_and(|current| current.sheet_index == *sheet_index) =>
            {
                if let Some((_, column)) = self.requested {
                    self.load_selection(column);
                }
            }
            _ => {}
        }
    }

    fn reset_request(&mut self) {
        self.requested = None;
        self.candidate_values.clear();
        self.selected_values.clear();
        self.include_blank = false;
        self.include_non_blank = false;
        self.loading = false;
        self.candidate_truncated = false;
        self.has_existing_filter = false;
    }

    fn request(&mut self, sheet_index: usize, column: usize) {
        self.requested = Some((sheet_index, column));
        self.candidate_values.clear();
        self.loading = true;
        self.candidate_truncated = false;
        self.load_selection(column);
    }

    fn load_selection(&mut self, column: usize) {
        self.selected_values.clear();
        self.include_blank = false;
        self.include_non_blank = false;
        self.has_existing_filter = false;
        let criteria = self.criteria_for_column(column);
        self.apply_criteria(&criteria);
    }

    fn criteria_for_column(&self, column: usize) -> Vec<SpreadsheetFilterCriterion> {
        self.metadata
            .as_ref()
            .and_then(|metadata| metadata.auto_filter.as_ref())
            .and_then(|filter| filter.columns.iter().find(|item| item.column == column))
            .map(|item| item.criteria.clone())
            .unwrap_or_default()
    }

    fn apply_criteria(&mut self, criteria: &[SpreadsheetFilterCriterion]) {
        self.has_existing_filter = !criteria.is_empty();
        for criterion in criteria {
            match criterion {
                SpreadsheetFilterCriterion::Values(values) => self.apply_values(values),
                SpreadsheetFilterCriterion::Blank => self.include_blank = true,
                SpreadsheetFilterCriterion::NonBlank => self.include_non_blank = true,
                SpreadsheetFilterCriterion::Unsupported(_) => {}
            }
        }
    }

    fn apply_values(&mut self, values: &[String]) {
        for value in values {
            if value.is_empty() {
                self.include_blank = true;
            } else {
                self.selected_values.insert(value.clone());
            }
        }
    }
}

pub(super) fn show(
    ui: &mut egui::Ui,
    viewport: egui::Rect,
    grid: &DocumentGridSurfaceFrame,
    state: &mut SpreadsheetFilterUiState,
) -> Vec<SpreadsheetFilterCommand> {
    let (sheet_index, range) = match state.metadata.as_ref() {
        Some(metadata) => match metadata.auto_filter.as_ref() {
            Some(filter) => (metadata.sheet_index, filter.range),
            None => return Vec::new(),
        },
        None => return Vec::new(),
    };
    let messages = &crate::i18n::I18nOps::get().preview.spreadsheet_filter;
    let mut commands = Vec::new();

    for cell in grid
        .cells
        .iter()
        .filter(|cell| eligible_header(cell, range))
    {
        let cell_rect =
            super::painter_grid_style::translated(viewport.min, cell.bounds).intersect(viewport);
        if cell_rect.is_negative() || cell_rect.width() <= FILTER_BUTTON_WIDTH {
            continue;
        }
        let button_rect = egui::Rect::from_min_max(
            egui::pos2(cell_rect.right() - FILTER_BUTTON_WIDTH, cell_rect.top()),
            egui::pos2(cell_rect.right(), cell_rect.bottom()),
        );
        let column = cell.coordinate.column;
        let response = crate::widgets::MenuButtonOps::show_unframed_interactive(
            &mut ui.new_child(egui::UiBuilder::new().max_rect(button_rect)),
            "▾",
            |menu| filter_menu(menu, messages, state, sheet_index, column),
        );
        if response.response.clicked() {
            state.request(sheet_index, column);
            commands.push(SpreadsheetFilterCommand::Candidates {
                sheet_index,
                column,
                limit: CANDIDATE_LIMIT,
            });
        }
        if let Some(Some(command)) = response.inner {
            commands.push(command);
        }
        let accessibility_label = format!("{} {}", messages.open, column + 1);
        response.response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, accessibility_label.clone())
        });
        response.response.on_hover_text(accessibility_label);
    }
    commands
}

fn eligible_header(
    cell: &DocumentGridCell,
    range: katana_document_viewer::SpreadsheetFilterRange,
) -> bool {
    cell.coordinate.row == range.start.row
        && (range.start.column..=range.end.column).contains(&cell.coordinate.column)
}

fn filter_menu(
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
                let mut selected = state.selected_values.contains(&value);
                let was_selected = selected;
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .left(|ui| ui.label(&value))
                    .right(|ui| crate::widgets::ToggleOps::switch(ui, &mut selected))
                    .show(ui);
                if selected != was_selected {
                    if selected {
                        state.selected_values.insert(value.clone());
                    } else {
                        state.selected_values.remove(&value);
                    }
                }
            }
        });
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
            let response = ui.add_enabled(
                !state.candidate_truncated,
                egui::Button::new(&messages.apply),
            );
            if response.clicked() {
                command = Some(SpreadsheetFilterCommand::ApplyValues {
                    sheet_index,
                    column,
                    values: selected_filter_values(state),
                });
                ui.close();
            }
            ui.add_space(ACTION_BUTTON_SPACING);
            let response = ui.button(&messages.clear);
            if response.clicked() {
                command = Some(SpreadsheetFilterCommand::Clear {
                    sheet_index,
                    column: Some(column),
                });
                state.selected_values.clear();
                state.include_blank = false;
                state.include_non_blank = false;
                ui.close();
            }
        })
        .show(ui);
    command
}

fn selected_filter_values(state: &SpreadsheetFilterUiState) -> Vec<String> {
    let mut values: Vec<String> = state
        .selected_values
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect();
    if state.include_blank {
        values.push(String::new());
    }
    values
}
