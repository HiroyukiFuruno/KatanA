use eframe::egui;
use katana_document_viewer::{
    DocumentGridCommand, DocumentGridNavigation, DocumentGridSurfaceFrame, DocumentSurfaceCommand,
    DocumentSurfaceFrame, DocumentViewport,
};

pub(super) fn paint(
    ui: &mut egui::Ui,
    frame: &DocumentSurfaceFrame,
) -> Vec<DocumentSurfaceCommand> {
    paint_with_filters(ui, frame, &mut Default::default()).0
}

pub(super) fn paint_with_filters(
    ui: &mut egui::Ui,
    frame: &DocumentSurfaceFrame,
    filters: &mut super::spreadsheet_filter_controls::SpreadsheetFilterUiState,
) -> (
    Vec<DocumentSurfaceCommand>,
    Vec<katana_document_viewer::SpreadsheetFilterCommand>,
) {
    let size = ui.available_size().max(egui::vec2(1.0, 1.0));
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let Some(grid) = frame.grid() else {
        return (vec![resize_command(rect)], Vec::new());
    };
    super::painter_grid_style::paint_grid(ui, rect, grid);
    let commands = commands(ui, rect, &response, grid);
    let filter_commands = super::spreadsheet_filter_controls::show(ui, rect, grid, filters);
    (commands, filter_commands)
}

fn commands(
    ui: &egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    grid: &DocumentGridSurfaceFrame,
) -> Vec<DocumentSurfaceCommand> {
    let mut commands = vec![resize_command(rect)];
    commands.extend(selection_command(ui, rect, response));
    commands.extend(scroll_command(ui, response, grid));
    commands.extend(navigation_command(ui, response));
    commands
}

fn resize_command(rect: egui::Rect) -> DocumentSurfaceCommand {
    DocumentSurfaceCommand::Resize(DocumentViewport::new(
        rect.width() as u32,
        rect.height() as u32,
    ))
}

fn selection_command(
    ui: &egui::Ui,
    viewport: egui::Rect,
    response: &egui::Response,
) -> Option<DocumentSurfaceCommand> {
    if !response.clicked() {
        return None;
    }
    response.request_focus();
    let pointer = response.interact_pointer_pos()? - viewport.min;
    Some(DocumentSurfaceCommand::Grid(
        DocumentGridCommand::SelectAt {
            x: pointer.x.round() as i32,
            y: pointer.y.round() as i32,
            extend: ui.input(|input| input.modifiers.shift),
        },
    ))
}

fn scroll_command(
    ui: &egui::Ui,
    response: &egui::Response,
    grid: &DocumentGridSurfaceFrame,
) -> Option<DocumentSurfaceCommand> {
    if !response.hovered() {
        return None;
    }
    let delta = ui.input(|input| input.smooth_scroll_delta);
    let drag = if response.dragged() {
        ui.input(|input| input.pointer.delta())
    } else {
        egui::Vec2::ZERO
    };
    let movement = delta + drag;
    (movement != egui::Vec2::ZERO).then(|| scroll_to(grid, movement))
}

fn scroll_to(grid: &DocumentGridSurfaceFrame, movement: egui::Vec2) -> DocumentSurfaceCommand {
    DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo {
        x: offset(grid.scroll_x(), -movement.x),
        y: offset(grid.scroll_y(), -movement.y),
    })
}

fn navigation_command(ui: &egui::Ui, response: &egui::Response) -> Option<DocumentSurfaceCommand> {
    let intent = response
        .has_focus()
        .then(|| navigation_intent(ui))
        .flatten()?;
    Some(DocumentSurfaceCommand::Grid(
        DocumentGridCommand::Navigate {
            intent,
            extend: ui.input(|input| input.modifiers.shift),
        },
    ))
}

fn offset(current: u32, delta: f32) -> u32 {
    if delta >= 0.0 {
        current.saturating_add(delta.round() as u32)
    } else {
        current.saturating_sub((-delta).round() as u32)
    }
}

fn navigation_intent(ui: &egui::Ui) -> Option<DocumentGridNavigation> {
    let mappings = [
        (egui::Key::ArrowLeft, DocumentGridNavigation::Left),
        (egui::Key::ArrowRight, DocumentGridNavigation::Right),
        (egui::Key::ArrowUp, DocumentGridNavigation::Up),
        (egui::Key::ArrowDown, DocumentGridNavigation::Down),
        (egui::Key::Home, DocumentGridNavigation::Home),
        (egui::Key::End, DocumentGridNavigation::End),
        (egui::Key::PageUp, DocumentGridNavigation::PageUp),
        (egui::Key::PageDown, DocumentGridNavigation::PageDown),
    ];
    ui.input(|input| {
        mappings
            .into_iter()
            .find_map(|(key, intent)| input.key_pressed(key).then_some(intent))
    })
}

#[cfg(test)]
#[path = "painter_grid_tests.rs"]
mod tests;

#[cfg(test)]
mod filter_pointer_tests {
    use eframe::egui;
    use katana_document_viewer::{
        SpreadsheetAutoFilterArtifact, SpreadsheetCoordinate, SpreadsheetFilterColumnArtifact,
        SpreadsheetFilterCommand, SpreadsheetFilterRange, SpreadsheetFrameMetadata,
    };

    const TEST_VIEWPORT: egui::Vec2 = egui::vec2(300.0, 200.0);
    const FILTER_SHEET: usize = 0;
    const FILTER_COLUMN: usize = 0;
    const FILTER_LIMIT: usize = 512;

    fn metadata() -> SpreadsheetFrameMetadata {
        SpreadsheetFrameMetadata {
            sheet_index: FILTER_SHEET,
            visible_row_count: 1,
            auto_filter: Some(SpreadsheetAutoFilterArtifact {
                range: SpreadsheetFilterRange {
                    start: SpreadsheetCoordinate::new(0, 0),
                    end: SpreadsheetCoordinate::new(0, 0),
                },
                columns: vec![SpreadsheetFilterColumnArtifact {
                    column: FILTER_COLUMN,
                    criteria: Vec::new(),
                    candidates: Vec::new(),
                }],
                filtered_out_rows: Vec::new(),
                diagnostics: Vec::new(),
            }),
        }
    }

    fn pointer_input(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }

    #[test]
    fn filter_header_click_does_not_select_a_grid_cell() {
        let context = crate::test_ui::Context::default();
        let grid = super::super::painter_tests::grid_surface();
        let mut filter_state =
            super::super::spreadsheet_filter_controls::SpreadsheetFilterUiState::default();
        filter_state.update_metadata(Some(metadata()), None);
        let pointer = egui::pos2(120.0, 20.0);
        let input = |events| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, TEST_VIEWPORT)),
            events,
            ..Default::default()
        };

        for events in [
            vec![egui::Event::PointerMoved(pointer)],
            vec![pointer_input(pointer, true)],
            vec![pointer_input(pointer, false)],
        ] {
            let mut grid_commands = Vec::new();
            let mut filter_commands = Vec::new();
            context.run_ui(input(events), |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                grid_commands = super::commands(ui, rect, &response, &grid);
                filter_commands = super::super::spreadsheet_filter_controls::show(
                    ui,
                    rect,
                    &grid,
                    &mut filter_state,
                );
            });
            if filter_commands.iter().any(|command| {
                matches!(
                    command,
                    SpreadsheetFilterCommand::Candidates {
                        sheet_index: FILTER_SHEET,
                        column: FILTER_COLUMN,
                        limit: FILTER_LIMIT,
                    }
                )
            }) {
                assert!(!grid_commands.iter().any(|command| {
                    matches!(
                        command,
                        katana_document_viewer::DocumentSurfaceCommand::Grid(
                            katana_document_viewer::DocumentGridCommand::SelectAt { .. }
                        )
                    )
                }));
                return;
            }
        }

        panic!("filter header click did not request candidates");
    }
}
