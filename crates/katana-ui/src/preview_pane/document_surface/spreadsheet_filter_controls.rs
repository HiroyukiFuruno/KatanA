#[cfg(test)]
#[path = "spreadsheet_filter_controls_tests.rs"]
mod tests;

#[path = "spreadsheet_filter_controls_state.rs"]
mod filter_state;

#[path = "spreadsheet_filter_controls_menu.rs"]
mod menu;

#[cfg(test)]
pub(super) use filter_state::selected_filter_values;
pub(super) use filter_state::{SpreadsheetFilterUiState, apply_command};
use menu::filter_menu;

use eframe::egui;
use katana_document_viewer::{
    DocumentGridCell, DocumentGridSurfaceFrame, SpreadsheetFilterCommand,
};

const CANDIDATE_LIMIT: usize = 512;
const FILTER_BUTTON_WIDTH: f32 = 22.0;

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
        commands.extend(show_header_filter(
            ui,
            viewport,
            cell,
            messages,
            state,
            sheet_index,
        ));
    }
    commands
}

fn show_header_filter(
    ui: &mut egui::Ui,
    viewport: egui::Rect,
    cell: &DocumentGridCell,
    messages: &crate::i18n::SpreadsheetFilterMessages,
    state: &mut SpreadsheetFilterUiState,
    sheet_index: usize,
) -> Vec<SpreadsheetFilterCommand> {
    let Some((button_rect, cell_clip)) = filter_button_rect(viewport, cell) else {
        return Vec::new();
    };
    let column = cell.coordinate.column;
    let mut button_ui = ui.new_child(egui::UiBuilder::new().max_rect(button_rect));
    button_ui.shrink_clip_rect(cell_clip);
    let response =
        crate::widgets::MenuButtonOps::show_unframed_interactive(&mut button_ui, "▾", |menu| {
            filter_menu(menu, messages, state, sheet_index, column)
        });
    let mut commands = Vec::new();
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
    show_accessibility_label(response.response, messages, column);
    commands
}

fn filter_button_rect(
    viewport: egui::Rect,
    cell: &DocumentGridCell,
) -> Option<(egui::Rect, egui::Rect)> {
    let cell_rect = super::painter_grid_style::translated(viewport.min, cell.bounds);
    if cell_rect.width() <= FILTER_BUTTON_WIDTH {
        return None;
    }
    let cell_clip = super::painter_grid_style::translated(viewport.min, cell.clipped_bounds)
        .intersect(viewport);
    if cell_clip.is_negative() || cell_clip.width() <= 0.0 || cell_clip.height() <= 0.0 {
        return None;
    }
    let button_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.right() - FILTER_BUTTON_WIDTH, cell_rect.top()),
        egui::pos2(cell_rect.right(), cell_rect.bottom()),
    );
    let visible_button = button_rect.intersect(cell_clip);
    (!visible_button.is_negative() && visible_button.width() > 0.0 && visible_button.height() > 0.0)
        .then_some((button_rect, cell_clip))
}

fn show_accessibility_label(
    response: egui::Response,
    messages: &crate::i18n::SpreadsheetFilterMessages,
    column: usize,
) {
    let label = format!("{} {}", messages.open, column + 1);
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone()));
    response.on_hover_text(label);
}

fn eligible_header(
    cell: &DocumentGridCell,
    range: katana_document_viewer::SpreadsheetFilterRange,
) -> bool {
    cell.coordinate.row == range.start.row
        && (range.start.column..=range.end.column).contains(&cell.coordinate.column)
}
