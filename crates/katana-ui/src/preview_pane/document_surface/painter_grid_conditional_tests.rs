use eframe::egui;
use katana_document_viewer::{DocumentGridDataBar, DocumentGridIcon, DocumentGridRating};

use super::super::painter_tests::grid_cell;

const CELL_WIDTH: f32 = 120.0;
const CELL_HEIGHT: f32 = 32.0;
const POSITIVE_FILL: u16 = 8_000;
const NEGATIVE_FILL: u16 = 2_000;
const AXIS: u16 = 5_000;
const RATING_COUNT: u32 = 50;
const RATING_MAXIMUM: u32 = 20;
const ABOVE_MAX_RATIO: u16 = 20_000;

#[test]
fn conditional_styles_cover_bars_icons_ratings_and_hidden_values() {
    let context = crate::test_ui::Context::default();
    context.run_ui(egui::RawInput::default(), |ui| {
        let painter = ui.painter();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(CELL_WIDTH, CELL_HEIGHT));

        let mut positive = grid_cell();
        positive.appearance.data_bar = Some(DocumentGridDataBar {
            positive_color: Some("#00ff00".to_owned()),
            negative_color: None,
            fill_ratio_basis_points: POSITIVE_FILL,
            axis_ratio_basis_points: AXIS,
            gradient: true,
            show_value: false,
        });
        assert_eq!(super::paint(painter, rect, rect, &positive, ui), 0.0);
        assert!(!super::show_cell_value(&positive));

        let mut negative = grid_cell();
        negative.appearance.data_bar = Some(DocumentGridDataBar {
            positive_color: None,
            negative_color: Some("invalid".to_owned()),
            fill_ratio_basis_points: NEGATIVE_FILL,
            axis_ratio_basis_points: AXIS,
            gradient: false,
            show_value: true,
        });
        super::paint(painter, rect, rect, &negative, ui);

        let mut icon = grid_cell();
        icon.appearance.icon = Some(DocumentGridIcon {
            name: "check".to_owned(),
            color: None,
            show_value: true,
        });
        assert!(super::paint(painter, rect, rect, &icon, ui) > 0.0);

        let mut rating = grid_cell();
        rating.appearance.rating = Some(DocumentGridRating {
            icon_name: "star".to_owned(),
            count: RATING_COUNT,
            maximum: RATING_MAXIMUM,
            color: Some("invalid".to_owned()),
            show_value: false,
        });
        assert!(super::paint(painter, rect, rect, &rating, ui) > 0.0);
        assert!(!super::show_cell_value(&rating));
    });

    assert_eq!(super::ratio(ABOVE_MAX_RATIO), 1.0);
    for (name, symbol) in [
        ("up", "\u{2191}"),
        ("down", "\u{2193}"),
        ("left", "\u{2190}"),
        ("right", "\u{2192}"),
        ("star", "\u{2605}"),
        ("check", "\u{2713}"),
        ("cross", "\u{00d7}"),
        ("xmark", "\u{00d7}"),
        ("flag", "\u{2691}"),
        ("unknown", "\u{25cf}"),
    ] {
        assert_eq!(super::icon_symbol(name), symbol);
    }
}
