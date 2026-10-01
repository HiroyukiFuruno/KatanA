use eframe::egui;
use katana_document_viewer::{DocumentGridHorizontalAlignment, DocumentGridVerticalAlignment};

use super::super::painter_tests::{grid_cell, grid_surface};

const TEXT_RECT_WIDTH: f32 = 140.0;
const TEXT_RECT_HEIGHT: f32 = 40.0;
const TOO_SMALL_RECT_WIDTH: f32 = 8.0;
const TOO_SMALL_RECT_HEIGHT: f32 = 4.0;

#[test]
fn grid_text_skips_when_cell_is_too_small_for_text_area() {
    let context = crate::test_ui::Context::default();
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(TOO_SMALL_RECT_WIDTH, TOO_SMALL_RECT_HEIGHT),
            )),
            ..Default::default()
        },
        |ui| {
            let rect = egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(TOO_SMALL_RECT_WIDTH, TOO_SMALL_RECT_HEIGHT),
            );
            super::paint_with_fonts(ui.painter(), rect, &grid_cell(), ui, 0.0, None);
        },
    );

    assert!(
        output
            .shapes
            .iter()
            .all(|shape| { !matches!(shape.shape, egui::Shape::Text(_)) })
    );
}

#[test]
fn grid_text_honors_registered_family_without_expanding_font_resources() {
    let context = crate::test_ui::Context::default();
    let mut definitions = egui::FontDefinitions::default();
    let registered = egui::FontFamily::Name("Office Fixture Sans".into());
    let proportional = definitions.families[&egui::FontFamily::Proportional].clone();
    definitions
        .families
        .insert(registered.clone(), proportional);
    let expected_data_count = definitions.font_data.len();
    let expected_family_count = definitions.families.len();
    context.set_fonts(definitions);
    for name in ["Office Fixture Sans", "office fixture sans"] {
        let mut cell = grid_cell();
        cell.appearance.font_family = name.to_owned();
        let galleys = paint_and_galleys(&context, &cell);
        assert_eq!(galleys[0].job.sections[0].format.font_id.family, registered);
        context.run_ui(egui::RawInput::default(), |ui| {
            ui.fonts(|fonts| {
                assert_eq!(fonts.definitions().font_data.len(), expected_data_count);
                assert_eq!(fonts.definitions().families.len(), expected_family_count);
            });
        });
    }
}

#[test]
fn grid_text_keeps_existing_fallback_for_unregistered_families() {
    let context = crate::test_ui::Context::default();
    for (name, expected) in [
        ("Uninstalled Office Sans", egui::FontFamily::Proportional),
        ("Uninstalled Office Mono", egui::FontFamily::Monospace),
    ] {
        let mut cell = grid_cell();
        cell.appearance.font_family = name.to_owned();
        let galleys = paint_and_galleys(&context, &cell);
        assert_eq!(galleys[0].job.sections[0].format.font_id.family, expected);
    }
}

#[test]
fn grid_text_covers_alignment_font_decoration_wrap_and_visibility() {
    let context = crate::test_ui::Context::default();
    context.run_ui(egui::RawInput::default(), |ui| {
        paint_empty_and_normal(ui);
        paint_alignment_cases(ui);
    });

    assert_eq!(
        super::horizontal_alignment(DocumentGridHorizontalAlignment::Right),
        egui::Align::RIGHT
    );
    assert_eq!(
        super::horizontal_alignment(DocumentGridHorizontalAlignment::Center),
        egui::Align::Center
    );
    assert_eq!(
        super::horizontal_alignment(DocumentGridHorizontalAlignment::Left),
        egui::Align::LEFT
    );
}

fn paint_empty_and_normal(ui: &egui::Ui) {
    let mut empty = grid_cell();
    empty.text.clear();
    paint_grid(ui, &empty);
    paint_grid(ui, &grid_cell());
}

fn paint_alignment_cases(ui: &egui::Ui) {
    for (horizontal, vertical) in [
        (
            DocumentGridHorizontalAlignment::Right,
            DocumentGridVerticalAlignment::Top,
        ),
        (
            DocumentGridHorizontalAlignment::Center,
            DocumentGridVerticalAlignment::Center,
        ),
        (
            DocumentGridHorizontalAlignment::Justify,
            DocumentGridVerticalAlignment::Bottom,
        ),
        (
            DocumentGridHorizontalAlignment::Distributed,
            DocumentGridVerticalAlignment::Distributed,
        ),
    ] {
        paint_styled_cell(ui, horizontal, vertical);
    }
}

fn paint_styled_cell(
    ui: &egui::Ui,
    horizontal: DocumentGridHorizontalAlignment,
    vertical: DocumentGridVerticalAlignment,
) {
    let mut cell = grid_cell();
    cell.appearance.horizontal_alignment = horizontal;
    cell.appearance.vertical_alignment = vertical;
    cell.appearance.font_family = "Consolas Mono".to_owned();
    cell.appearance.font_size_px = 0;
    cell.appearance.text_color = Some("invalid".to_owned());
    cell.appearance.bold = true;
    cell.appearance.italic = true;
    cell.appearance.underline = true;
    cell.appearance.strike = true;
    cell.appearance.wrap_text = true;
    paint_grid(ui, &cell);
}

fn paint_grid(ui: &egui::Ui, cell: &katana_document_viewer::DocumentGridCell) {
    let mut frame = grid_surface();
    frame.cells[0] = cell.clone();
    super::super::painter_grid_style::paint_grid(
        ui,
        egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(TEXT_RECT_WIDTH, TEXT_RECT_HEIGHT),
        ),
        &frame,
        None,
    );
}

fn paint_and_galleys(
    context: &crate::test_ui::Context,
    cell: &katana_document_viewer::DocumentGridCell,
) -> Vec<std::sync::Arc<egui::Galley>> {
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(TEXT_RECT_WIDTH, TEXT_RECT_HEIGHT),
            )),
            ..Default::default()
        },
        |ui| paint_grid(ui, cell),
    );
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.clone()),
            _ => None,
        })
        .collect()
}
