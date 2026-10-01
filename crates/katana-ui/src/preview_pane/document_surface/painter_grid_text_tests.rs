use eframe::egui;
use katana_document_viewer::{DocumentGridHorizontalAlignment, DocumentGridVerticalAlignment};

use super::super::painter_tests::grid_cell;

const TEXT_RECT_WIDTH: f32 = 140.0;
const TEXT_RECT_HEIGHT: f32 = 40.0;
const HIDDEN_TEXT_INDICATOR_WIDTH: f32 = 1_000.0;

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
        context.run_ui(egui::RawInput::default(), |ui| {
            let mut cell = grid_cell();
            cell.appearance.font_family = name.to_owned();
            let galley = super::layout_text(ui, &cell, TEXT_RECT_WIDTH, ui.visuals().text_color());
            assert_eq!(galley.job.sections[0].format.font_id.family, registered);
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
    context.run_ui(egui::RawInput::default(), |ui| {
        for (name, expected) in [
            ("Uninstalled Office Sans", egui::FontFamily::Proportional),
            ("Uninstalled Office Mono", egui::FontFamily::Monospace),
        ] {
            let mut cell = grid_cell();
            cell.appearance.font_family = name.to_owned();
            let galley = super::layout_text(ui, &cell, TEXT_RECT_WIDTH, ui.visuals().text_color());
            assert_eq!(galley.job.sections[0].format.font_id.family, expected);
        }
    });
}

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
