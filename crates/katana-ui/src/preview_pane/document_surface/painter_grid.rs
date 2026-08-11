use eframe::egui;
use katana_document_viewer::{
    DocumentGridCommand, DocumentGridNavigation, DocumentGridSurfaceFrame, DocumentSurfaceCommand,
    DocumentSurfaceFrame, DocumentViewport,
};

pub(super) fn paint(
    ui: &mut egui::Ui,
    frame: &DocumentSurfaceFrame,
) -> Vec<DocumentSurfaceCommand> {
    let size = ui.available_size().max(egui::vec2(1.0, 1.0));
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let Some(grid) = frame.grid() else {
        return vec![resize_command(rect)];
    };
    super::painter_grid_style::paint_grid(ui, rect, grid);
    commands(ui, rect, &response, grid)
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
