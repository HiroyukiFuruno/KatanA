use eframe::egui;
use katana_document_viewer::{DocumentGridHorizontalAlignment, DocumentGridVerticalAlignment};

use super::super::painter_tests::grid_cell;

const TEXT_RECT_WIDTH: f32 = 140.0;
const TEXT_RECT_HEIGHT: f32 = 40.0;
const HIDDEN_TEXT_INDICATOR_WIDTH: f32 = 1_000.0;

#[test]
fn grid_text_covers_alignment_font_decoration_wrap_and_visibility() {
    let context = crate::test_ui::Context::default();
    context.run_ui(egui::RawInput::default(), |ui| {
        let painter = ui.painter();
        let rect = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(TEXT_RECT_WIDTH, TEXT_RECT_HEIGHT),
        );

        let mut empty = grid_cell();
        empty.text.clear();
        super::paint(painter, rect, &empty, ui, 0.0);
        super::paint(painter, rect, &grid_cell(), ui, HIDDEN_TEXT_INDICATOR_WIDTH);

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
            super::paint(painter, rect, &cell, ui, 0.0);
        }
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
