use eframe::egui;
use katana_document_viewer::{DocumentGridCell, DocumentGridSurfaceFrame, DocumentRect};

const ACTIVE_SELECTION_OPACITY: f32 = 0.45;
const SELECTION_OPACITY: f32 = 0.25;
const GRID_LINE_WIDTH: f32 = 1.0;

pub(super) fn paint_grid(ui: &egui::Ui, viewport: egui::Rect, frame: &DocumentGridSurfaceFrame) {
    let painter = ui.painter().with_clip_rect(viewport);
    painter.rect_filled(viewport, 0.0, ui.visuals().extreme_bg_color);
    for cell in &frame.cells {
        paint_cell(ui, &painter, viewport, cell, frame.show_grid_lines);
    }
}

fn paint_cell(
    ui: &egui::Ui,
    painter: &egui::Painter,
    viewport: egui::Rect,
    cell: &DocumentGridCell,
    show_grid_lines: bool,
) {
    let rect = translated(viewport.min, cell.bounds);
    let clip = translated(viewport.min, cell.clipped_bounds).intersect(viewport);
    if clip.is_negative() || clip.width() <= 0.0 || clip.height() <= 0.0 {
        return;
    }
    let painter = painter.with_clip_rect(clip);
    painter.rect_filled(rect, 0.0, fill_color(ui, cell));
    let indicator_width = super::painter_grid_conditional::paint(&painter, rect, clip, cell, ui);
    paint_selection(ui, &painter, rect, cell);
    paint_grid_line(ui, &painter, rect, show_grid_lines);
    super::painter_grid_text::paint(&painter, rect, cell, ui, indicator_width);
}

fn fill_color(ui: &egui::Ui, cell: &DocumentGridCell) -> egui::Color32 {
    cell.appearance
        .fill_color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or_else(|| ui.visuals().faint_bg_color)
}

fn paint_selection(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    cell: &DocumentGridCell,
) {
    if cell.selected || cell.active {
        let opacity = if cell.active {
            ACTIVE_SELECTION_OPACITY
        } else {
            SELECTION_OPACITY
        };
        let color = ui.visuals().selection.bg_fill.gamma_multiply(opacity);
        painter.rect_filled(rect, 0.0, color);
    }
}

fn paint_grid_line(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    show_grid_lines: bool,
) {
    if show_grid_lines {
        let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let stroke = egui::Stroke::new(GRID_LINE_WIDTH, color);
        painter.add(egui::Shape::rect_stroke(
            rect.shrink(GRID_LINE_WIDTH),
            0.0,
            stroke,
            egui::StrokeKind::Inside,
        ));
    }
}

pub(super) fn translated(origin: egui::Pos2, rect: DocumentRect) -> egui::Rect {
    egui::Rect::from_min_size(
        origin + egui::vec2(rect.x as f32, rect.y as f32),
        egui::vec2(rect.width as f32, rect.height as f32),
    )
}

pub(super) fn parse_color(value: &str) -> Option<egui::Color32> {
    egui::Color32::from_hex(value).ok()
}

#[cfg(test)]
#[path = "painter_grid_style_tests.rs"]
mod tests;
