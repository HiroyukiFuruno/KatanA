#![cfg(test)]

use eframe::egui;
use katana_document_viewer::DocumentRect;

use super::{SpreadsheetFilterUiState, metadata};

const VIEWPORT: egui::Vec2 = egui::vec2(300.0, 100.0);
const CELL_LEFT: i32 = 250;
const CELL_TOP: i32 = 10;
const FULL_CELL_WIDTH: u32 = 120;
const CLIPPED_CELL_WIDTH: u32 = 50;
const CELL_HEIGHT: u32 = 24;
const VIEWPORT_EDGE_CLICK_X: f32 = 290.0;
const CLICK_Y: f32 = 20.0;

#[test]
fn partially_scrolled_header_does_not_move_filter_hit_target_to_viewport_edge() {
    let context = crate::test_ui::Context::default();
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(Some(metadata(0, Vec::new())), None);
    let grid = partially_clipped_grid();
    let commands = click_at_viewport_edge(&context, &mut state, &grid);
    assert!(
        commands.is_empty(),
        "offscreen anchor must not be clickable"
    );
}

fn partially_clipped_grid() -> katana_document_viewer::DocumentGridSurfaceFrame {
    let mut grid = super::super::super::painter_tests::grid_surface();
    let cell = &mut grid.cells[0];
    cell.bounds = DocumentRect {
        x: CELL_LEFT,
        y: CELL_TOP,
        width: FULL_CELL_WIDTH,
        height: CELL_HEIGHT,
    };
    cell.clipped_bounds = DocumentRect {
        x: CELL_LEFT,
        y: CELL_TOP,
        width: CLIPPED_CELL_WIDTH,
        height: CELL_HEIGHT,
    };
    grid
}

fn click_at_viewport_edge(
    context: &crate::test_ui::Context,
    state: &mut SpreadsheetFilterUiState,
    grid: &katana_document_viewer::DocumentGridSurfaceFrame,
) -> Vec<katana_document_viewer::SpreadsheetFilterCommand> {
    let pointer = egui::pos2(VIEWPORT_EDGE_CLICK_X, CLICK_Y);
    let mut commands = Vec::new();
    for event in [
        egui::Event::PointerMoved(pointer),
        pointer_button(pointer, true),
        pointer_button(pointer, false),
    ] {
        commands.extend(run_event(context, state, grid, event));
    }
    commands
}

fn pointer_button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

fn run_event(
    context: &crate::test_ui::Context,
    state: &mut SpreadsheetFilterUiState,
    grid: &katana_document_viewer::DocumentGridSurfaceFrame,
    event: egui::Event,
) -> Vec<katana_document_viewer::SpreadsheetFilterCommand> {
    let mut commands = Vec::new();
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, VIEWPORT)),
            events: vec![event],
            ..Default::default()
        },
        |ui| {
            commands = super::super::show(
                ui,
                egui::Rect::from_min_size(egui::Pos2::ZERO, VIEWPORT),
                grid,
                state,
            );
        },
    );
    commands
}
