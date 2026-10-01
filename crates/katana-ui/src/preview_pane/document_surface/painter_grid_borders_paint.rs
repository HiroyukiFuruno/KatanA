use eframe::egui;
use katana_document_viewer::DocumentGridSurfaceFrame;

use super::painter_grid_borders::{PreparedBorderSide, PreparedBorderStyle, PreparedGridBorders};

const DASH_LENGTH: f32 = 6.0;
const GAP_LENGTH: f32 = 3.0;
const DOT_LENGTH: f32 = 1.0;
const DOUBLE_STROKE_WIDTH: f32 = 1.0;
const DOUBLE_EDGE_OFFSET: f32 = 0.5;
const DOUBLE_INNER_OFFSET: f32 = 2.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BorderPaintError {
    CountMismatch(usize, usize),
    CoordinateMismatch,
}

impl std::fmt::Display for BorderPaintError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CountMismatch(grid, prepared) => write!(
                formatter,
                "prepared border count {prepared} does not match grid count {grid}"
            ),
            Self::CoordinateMismatch => write!(
                formatter,
                "prepared border coordinate does not match grid cell"
            ),
        }
    }
}

pub(super) fn paint_prepared(
    ui: &egui::Ui,
    viewport: egui::Rect,
    grid: &DocumentGridSurfaceFrame,
    prepared: &PreparedGridBorders,
) -> Result<(), BorderPaintError> {
    if grid.cells.len() != prepared.cells.len() {
        return Err(BorderPaintError::CountMismatch(
            grid.cells.len(),
            prepared.cells.len(),
        ));
    }
    for (cell, borders) in grid.cells.iter().zip(&prepared.cells) {
        if cell.coordinate != borders.coordinate {
            return Err(BorderPaintError::CoordinateMismatch);
        }
        let rect = super::painter_grid_style::translated(viewport.min, cell.bounds);
        let clip = super::painter_grid_style::translated(viewport.min, cell.clipped_bounds)
            .intersect(viewport);
        if clip.is_negative() || clip.width() <= 0.0 || clip.height() <= 0.0 {
            continue;
        }
        let painter = ui.painter().with_clip_rect(clip);
        let fallback = ui.visuals().widgets.noninteractive.bg_stroke.color;
        for (side, horizontal, far_edge) in [
            (borders.left.as_ref(), false, false),
            (borders.right.as_ref(), false, true),
            (borders.top.as_ref(), true, false),
            (borders.bottom.as_ref(), true, true),
        ] {
            paint_side(&painter, rect, side, horizontal, far_edge, fallback);
        }
    }
    Ok(())
}

fn paint_side(
    painter: &egui::Painter,
    rect: egui::Rect,
    side: Option<&PreparedBorderSide>,
    horizontal: bool,
    far_edge: bool,
    fallback: egui::Color32,
) {
    let Some(side) = side else { return };
    let color = side.color.unwrap_or(fallback);
    let width = side.style.width();
    let inward = if far_edge { -1.0 } else { 1.0 };
    let edge = if horizontal {
        let y = if far_edge { rect.bottom() } else { rect.top() };
        (egui::pos2(rect.left(), y), egui::pos2(rect.right(), y))
    } else {
        let x = if far_edge { rect.right() } else { rect.left() };
        (egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom()))
    };
    if side.style == PreparedBorderStyle::Double {
        let stroke = egui::Stroke::new(DOUBLE_STROKE_WIDTH, color);
        for distance in [DOUBLE_EDGE_OFFSET, DOUBLE_INNER_OFFSET] {
            let delta = if horizontal {
                egui::vec2(0.0, inward * distance)
            } else {
                egui::vec2(inward * distance, 0.0)
            };
            painter.line_segment([edge.0 + delta, edge.1 + delta], stroke);
        }
        return;
    }
    let inset = if horizontal {
        egui::vec2(0.0, inward * width / 2.0)
    } else {
        egui::vec2(inward * width / 2.0, 0.0)
    };
    let line = (edge.0 + inset, edge.1 + inset);
    emit_style(
        painter,
        line.0,
        line.1,
        side.style,
        egui::Stroke::new(width, color),
    );
}

fn emit_style(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    style: PreparedBorderStyle,
    stroke: egui::Stroke,
) {
    let pattern = match style {
        PreparedBorderStyle::Dotted => &[DOT_LENGTH, GAP_LENGTH][..],
        PreparedBorderStyle::Dashed | PreparedBorderStyle::MediumDashed => {
            &[DASH_LENGTH, GAP_LENGTH][..]
        }
        PreparedBorderStyle::DashDot | PreparedBorderStyle::MediumDashDot => {
            &[DASH_LENGTH, GAP_LENGTH, DOT_LENGTH, GAP_LENGTH][..]
        }
        PreparedBorderStyle::DashDotDot | PreparedBorderStyle::MediumDashDotDot => &[
            DASH_LENGTH,
            GAP_LENGTH,
            DOT_LENGTH,
            GAP_LENGTH,
            DOT_LENGTH,
            GAP_LENGTH,
        ][..],
        PreparedBorderStyle::SlantDashDot => {
            emit_pattern(
                painter,
                start,
                end,
                &[DASH_LENGTH, GAP_LENGTH, DOT_LENGTH, GAP_LENGTH],
                stroke,
                true,
            );
            return;
        }
        _ => {
            painter.line_segment([start, end], stroke);
            return;
        }
    };
    emit_pattern(painter, start, end, pattern, stroke, false);
}

fn emit_pattern(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    pattern: &[f32],
    stroke: egui::Stroke,
    slant: bool,
) {
    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }
    let direction = delta / length;
    let normal = egui::vec2(-direction.y, direction.x);
    let mut offset = 0.0;
    let mut index = 0;
    let mut draw = true;
    while offset < length {
        let next = (offset + pattern[index]).min(length);
        if draw {
            let slant_end = if slant { normal } else { egui::Vec2::ZERO };
            painter.line_segment(
                [
                    start + direction * offset,
                    start + direction * next + slant_end,
                ],
                stroke,
            );
        }
        offset = next;
        index = (index + 1) % pattern.len();
        draw = !draw;
    }
}

#[cfg(test)]
#[path = "painter_grid_borders_paint_tests.rs"]
mod tests;
