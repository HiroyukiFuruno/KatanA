use std::sync::Arc;

use crate::font_loader::office_font_leases::DocumentFontLease;
use eframe::egui;
use katana_document_viewer::{
    DocumentGridCell, DocumentGridHorizontalAlignment, DocumentGridVerticalAlignment,
};

use super::painter_grid_conditional::show_cell_value;
use super::painter_grid_style::parse_color;

const HORIZONTAL_PADDING: f32 = 5.0;
const VERTICAL_PADDING: f32 = 3.0;
const CENTER_RATIO: f32 = 0.5;
const BOLD_OFFSET: f32 = 0.55;

pub(super) fn paint_with_fonts(
    painter: &egui::Painter,
    rect: egui::Rect,
    cell: &DocumentGridCell,
    ui: &egui::Ui,
    indicator_width: f32,
    fonts: Option<&DocumentFontLease>,
) {
    if cell.text.is_empty() || !show_cell_value(cell) {
        return;
    }
    let Some(text_rect) = text_rect(rect, indicator_width) else {
        return;
    };
    let color = text_color(cell, ui);
    let (galley, faux_bold) = layout_text_with_fonts(ui, cell, text_rect.width(), color, fonts);
    let position = text_position(text_rect, galley.size(), cell);
    paint_galley(painter, position, galley, color, faux_bold);
}

fn text_rect(rect: egui::Rect, indicator_width: f32) -> Option<egui::Rect> {
    let min = egui::pos2(
        rect.left() + HORIZONTAL_PADDING + indicator_width,
        rect.top() + VERTICAL_PADDING,
    );
    let max = egui::pos2(
        rect.right() - HORIZONTAL_PADDING,
        rect.bottom() - VERTICAL_PADDING,
    );
    let result = egui::Rect::from_min_max(min, max);
    (result.width() > 0.0 && result.height() > 0.0).then_some(result)
}

fn text_color(cell: &DocumentGridCell, ui: &egui::Ui) -> egui::Color32 {
    cell.appearance
        .text_color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or_else(|| ui.visuals().text_color())
}

fn layout_text_with_fonts(
    ui: &egui::Ui,
    cell: &DocumentGridCell,
    max_width: f32,
    color: egui::Color32,
    fonts: Option<&DocumentFontLease>,
) -> (Arc<egui::Galley>, bool) {
    let style = text_style::style_for_cell(ui, fonts, cell);
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = if cell.appearance.wrap_text {
        max_width
    } else {
        f32::INFINITY
    };
    job.halign = horizontal_alignment(cell.appearance.horizontal_alignment);
    job.justify = matches!(
        cell.appearance.horizontal_alignment,
        DocumentGridHorizontalAlignment::Justify | DocumentGridHorizontalAlignment::Distributed
    );
    job.append(
        &cell.text,
        0.0,
        text_style::text_format(ui, cell, color, &style),
    );
    (ui.fonts_mut(|fonts| fonts.layout_job(job)), style.faux_bold)
}

fn horizontal_alignment(alignment: DocumentGridHorizontalAlignment) -> egui::Align {
    match alignment {
        DocumentGridHorizontalAlignment::Right => egui::Align::RIGHT,
        DocumentGridHorizontalAlignment::Center => egui::Align::Center,
        _ => egui::Align::LEFT,
    }
}

fn text_position(rect: egui::Rect, size: egui::Vec2, cell: &DocumentGridCell) -> egui::Pos2 {
    let x = match cell.appearance.horizontal_alignment {
        DocumentGridHorizontalAlignment::Right => rect.right() - size.x,
        DocumentGridHorizontalAlignment::Center => rect.center().x - size.x * CENTER_RATIO,
        _ => rect.left(),
    };
    let y = match cell.appearance.vertical_alignment {
        DocumentGridVerticalAlignment::Top => rect.top(),
        DocumentGridVerticalAlignment::Bottom => rect.bottom() - size.y,
        _ => rect.center().y - size.y * CENTER_RATIO,
    };
    egui::pos2(x, y)
}

fn paint_galley(
    painter: &egui::Painter,
    position: egui::Pos2,
    galley: Arc<egui::Galley>,
    color: egui::Color32,
    bold: bool,
) {
    painter.galley(position, galley.clone(), color);
    if bold {
        painter.galley(position + egui::vec2(BOLD_OFFSET, 0.0), galley, color);
    }
}

#[cfg(test)]
#[path = "painter_grid_text_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "painter_grid_font_lease_tests.rs"]
mod font_lease_tests;

#[cfg(test)]
#[path = "painter_grid_font_italic_tests.rs"]
mod font_italic_tests;

#[cfg(test)]
#[path = "painter_grid_font_test_support.rs"]
mod font_test_support;

#[path = "painter_grid_text_style.rs"]
mod text_style;
