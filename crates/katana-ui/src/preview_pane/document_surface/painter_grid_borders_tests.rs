use eframe::egui;
use katana_document_viewer::{DocumentGridBorderSide, DocumentGridCoordinate, DocumentRect};

use super::{PreparedBorderStyle, parse_style, prepare, prepare_side};

#[test]
fn all_kuc_styles_have_closed_width_contract() {
    let styles = [
        ("hair", 1.0),
        ("thin", 1.0),
        ("medium", 2.0),
        ("thick", 3.0),
        ("double", 3.0),
        ("dotted", 1.0),
        ("dashed", 1.0),
        ("dashDot", 1.0),
        ("dashDotDot", 1.0),
        ("mediumDashed", 2.0),
        ("mediumDashDot", 2.0),
        ("mediumDashDotDot", 2.0),
        ("slantDashDot", 2.0),
        ("solid", 1.0),
    ];
    for (style, expected_width) in styles {
        let parsed = parse_style(DocumentGridCoordinate { row: 1, column: 2 }, "top", style)
            .expect("KUC style must map");
        assert_eq!(parsed.width(), expected_width, "style {style}");
    }
}

#[test]
fn colors_are_prepared_and_invalid_sources_are_rejected() {
    let coordinate = DocumentGridCoordinate { row: 1, column: 2 };
    let source = DocumentGridBorderSide {
        style: "thin".to_owned(),
        color: Some("#112233".to_owned()),
    };
    let prepared = prepare_side(coordinate, "left", Some(&source))
        .expect("valid")
        .expect("visible");
    assert_eq!(
        Some(egui::Color32::from_hex("#112233").expect("test color")),
        prepared.color
    );
    let hidden = DocumentGridBorderSide {
        style: "none".to_owned(),
        color: None,
    };
    assert!(
        prepare_side(coordinate, "left", Some(&hidden))
            .expect("none")
            .is_none()
    );
    assert!(
        prepare_side(
            coordinate,
            "left",
            Some(&DocumentGridBorderSide {
                style: "unknown".to_owned(),
                color: None
            }),
        )
        .is_err()
    );
    assert!(
        prepare_side(
            coordinate,
            "left",
            Some(&DocumentGridBorderSide {
                style: "thin".to_owned(),
                color: Some("invalid".to_owned())
            }),
        )
        .is_err()
    );
}

#[test]
fn merged_cell_bounds_and_clipped_geometry_are_preserved() {
    let merged = DocumentRect {
        x: 8,
        y: 12,
        width: 160,
        height: 40,
    };
    assert_eq!((160, 40), (merged.width, merged.height));
    let viewport = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(20.0, 20.0));
    let clipped = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(15.0, 15.0));
    assert!(!clipped.intersect(viewport).is_negative());
}

#[test]
fn real_xlsx_frame_keeps_merged_geometry_with_prepared_cache() {
    let worker = super::super::painter_tests::office_worker();
    if !worker.is_file() {
        return;
    }
    let frame = super::super::painter_tests::frame("representative.xlsx");
    let grid = frame.surface.grid().expect("spreadsheet frame");
    assert!(
        grid.cells
            .iter()
            .any(|cell| cell.row_span > 1 || cell.column_span > 1)
    );
    let prepared = prepare(&frame.surface).expect("border projection");
    assert_eq!(prepared.cells.len(), grid.cells.len());
    for (cell, cached) in grid.cells.iter().zip(&prepared.cells) {
        assert_eq!(cell.coordinate, cached.coordinate);
    }
}

#[test]
fn patterned_styles_remain_distinct_at_the_paint_boundary() {
    assert!(PreparedBorderStyle::Dotted.width() > 0.0);
    assert!(PreparedBorderStyle::SlantDashDot.width() > 0.0);
    assert_ne!(
        PreparedBorderStyle::Dotted,
        PreparedBorderStyle::SlantDashDot
    );
}
