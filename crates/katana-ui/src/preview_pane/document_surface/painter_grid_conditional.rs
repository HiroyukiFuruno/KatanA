use eframe::egui;
use katana_document_viewer::{DocumentGridCell, DocumentGridDataBar};

use super::painter_grid_style::parse_color;

const INDICATOR_SIZE: f32 = 13.0;
const INDICATOR_GAP: f32 = 3.0;
const GRADIENT_OPACITY: f32 = 0.35;
const SOLID_OPACITY: f32 = 0.5;
const MAX_RATING_ICONS: u32 = 10;
const BASIS_POINTS_MAX: u16 = 10_000;

pub(super) fn paint(
    painter: &egui::Painter,
    rect: egui::Rect,
    clip: egui::Rect,
    cell: &DocumentGridCell,
    ui: &egui::Ui,
) -> f32 {
    paint_data_bar(painter, rect, clip, cell, ui);
    let Some((text, color)) = indicator(cell, ui) else {
        return 0.0;
    };
    painter.text(
        egui::pos2(rect.left() + INDICATOR_GAP, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &text,
        egui::FontId::proportional(INDICATOR_SIZE),
        color,
    );
    INDICATOR_SIZE * text.chars().count().max(1) as f32 + INDICATOR_GAP
}

pub(super) fn show_cell_value(cell: &DocumentGridCell) -> bool {
    cell.appearance
        .data_bar
        .as_ref()
        .is_none_or(|bar| bar.show_value)
        && cell
            .appearance
            .icon
            .as_ref()
            .is_none_or(|icon| icon.show_value)
        && cell
            .appearance
            .rating
            .as_ref()
            .is_none_or(|rating| rating.show_value)
}

fn paint_data_bar(
    painter: &egui::Painter,
    rect: egui::Rect,
    clip: egui::Rect,
    cell: &DocumentGridCell,
    ui: &egui::Ui,
) {
    let Some(bar) = &cell.appearance.data_bar else {
        return;
    };
    let (start, end, color) = data_bar_style(bar, ui);
    let min = egui::pos2(rect.left() + rect.width() * start, rect.top());
    let size = egui::vec2(rect.width() * (end - start), rect.height());
    let opacity = if bar.gradient {
        GRADIENT_OPACITY
    } else {
        SOLID_OPACITY
    };
    painter.rect_filled(
        egui::Rect::from_min_size(min, size).intersect(clip),
        0.0,
        color.gamma_multiply(opacity),
    );
}

fn data_bar_style(bar: &DocumentGridDataBar, ui: &egui::Ui) -> (f32, f32, egui::Color32) {
    let value = ratio(bar.fill_ratio_basis_points);
    let axis = ratio(bar.axis_ratio_basis_points);
    if value < axis {
        (
            value,
            axis,
            bar.negative_color
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(ui.visuals().error_fg_color),
        )
    } else {
        (
            axis,
            value,
            bar.positive_color
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(ui.visuals().selection.stroke.color),
        )
    }
}

fn indicator(cell: &DocumentGridCell, ui: &egui::Ui) -> Option<(String, egui::Color32)> {
    if let Some(icon) = &cell.appearance.icon {
        let color = icon
            .color
            .as_deref()
            .and_then(parse_color)
            .unwrap_or_else(|| ui.visuals().text_color());
        return Some((icon_symbol(&icon.name).to_owned(), color));
    }
    let rating = cell.appearance.rating.as_ref()?;
    let color = rating
        .color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(ui.visuals().warn_fg_color);
    let count = rating.count.min(rating.maximum).min(MAX_RATING_ICONS) as usize;
    Some((icon_symbol(&rating.icon_name).repeat(count), color))
}

fn ratio(basis_points: u16) -> f32 {
    f32::from(basis_points.min(BASIS_POINTS_MAX)) / f32::from(BASIS_POINTS_MAX)
}

fn icon_symbol(name: &str) -> &'static str {
    let normalized = name.to_ascii_lowercase();
    [
        ("up", "\u{2191}"),
        ("down", "\u{2193}"),
        ("left", "\u{2190}"),
        ("right", "\u{2192}"),
        ("star", "\u{2605}"),
        ("check", "\u{2713}"),
        ("cross", "\u{00d7}"),
        ("xmark", "\u{00d7}"),
        ("flag", "\u{2691}"),
    ]
    .into_iter()
    .find_map(|(needle, symbol)| normalized.contains(needle).then_some(symbol))
    .unwrap_or("\u{25cf}")
}

#[cfg(test)]
#[path = "painter_grid_conditional_tests.rs"]
mod tests;
