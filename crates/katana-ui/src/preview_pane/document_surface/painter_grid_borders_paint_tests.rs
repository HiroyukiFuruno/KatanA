use eframe::egui;
use katana_document_viewer::DocumentGridCoordinate;

use super::super::painter_grid_borders::{
    PreparedBorderSide, PreparedBorderStyle, PreparedCellBorders, PreparedGridBorders,
};
use super::{DOUBLE_INNER_OFFSET, DOUBLE_STROKE_WIDTH, emit_pattern, paint_prepared};

const VIEWPORT_WIDTH: f32 = 220.0;

fn side(style: &str, color: &str) -> PreparedBorderSide {
    let style = match style {
        "hair" => PreparedBorderStyle::Hair,
        "thin" => PreparedBorderStyle::Thin,
        "double" => PreparedBorderStyle::Double,
        "medium" => PreparedBorderStyle::Medium,
        "thick" => PreparedBorderStyle::Thick,
        "dotted" => PreparedBorderStyle::Dotted,
        "dashed" => PreparedBorderStyle::Dashed,
        "dashDot" => PreparedBorderStyle::DashDot,
        "dashDotDot" => PreparedBorderStyle::DashDotDot,
        "mediumDashed" => PreparedBorderStyle::MediumDashed,
        "mediumDashDot" => PreparedBorderStyle::MediumDashDot,
        "mediumDashDotDot" => PreparedBorderStyle::MediumDashDotDot,
        "slantDashDot" => PreparedBorderStyle::SlantDashDot,
        "solid" => PreparedBorderStyle::Solid,
        value => panic!("unexpected test style: {value}"),
    };
    PreparedBorderSide {
        style,
        color: Some(egui::Color32::from_hex(color).expect("test color")),
    }
}

fn painted_lines(style: &str) -> Vec<([egui::Pos2; 2], egui::Stroke)> {
    let context = crate::test_ui::Context::default();
    let grid = super::super::painter_tests::grid_surface();
    let prepared = PreparedGridBorders {
        cells: vec![PreparedCellBorders {
            coordinate: DocumentGridCoordinate { row: 0, column: 0 },
            left: None,
            right: None,
            top: Some(side(style, "#112233")),
            bottom: None,
        }],
    };
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(VIEWPORT_WIDTH, 100.0));
    let output = context.run_ui(egui::RawInput::default(), |ui| {
        paint_prepared(ui, viewport, &grid, &prepared).expect("paint style");
    });
    output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::LineSegment { points, stroke } => Some((*points, *stroke)),
            _ => None,
        })
        .collect()
}

#[test]
fn actual_paint_preserves_four_sides_width_color_gap_and_clip() {
    let context = crate::test_ui::Context::default();
    let grid = super::super::painter_tests::grid_surface();
    let prepared = PreparedGridBorders {
        cells: vec![PreparedCellBorders {
            coordinate: DocumentGridCoordinate { row: 0, column: 0 },
            left: Some(side("double", "#112233")),
            right: Some(side("medium", "#223344")),
            top: Some(side("thin", "#334455")),
            bottom: Some(side("dotted", "#445566")),
        }],
    };
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(VIEWPORT_WIDTH, 100.0));
    let output = context.run_ui(egui::RawInput::default(), |ui| {
        paint_prepared(ui, viewport, &grid, &prepared).expect("paint");
    });
    let lines: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::LineSegment { points, stroke } => {
                Some((clipped.clip_rect, *points, *stroke))
            }
            _ => None,
        })
        .collect();
    assert!(lines.len() >= 5);
    let double_color = egui::Color32::from_hex("#112233").expect("test color");
    let medium_color = egui::Color32::from_hex("#223344").expect("test color");
    assert!(lines.iter().any(|(_, points, stroke)| {
        stroke.color == double_color
            && (stroke.width - DOUBLE_STROKE_WIDTH).abs() < f32::EPSILON
            && (points[0].x - 10.5).abs() < f32::EPSILON
    }));
    assert!(lines.iter().any(|(_, _, stroke)| {
        stroke.color == medium_color
            && (stroke.width - PreparedBorderStyle::Medium.width()).abs() < f32::EPSILON
    }));
    let double_x: Vec<_> = lines
        .iter()
        .filter(|(_, points, stroke)| {
            stroke.color == double_color && (points[0].x - points[1].x).abs() < f32::EPSILON
        })
        .map(|(_, points, _)| points[0].x)
        .collect();
    assert!(double_x.len() >= 2);
    assert!(
        double_x
            .windows(2)
            .any(|pair| (pair[1] - pair[0]).abs() > DOUBLE_STROKE_WIDTH)
    );
    let expected_clip = egui::Rect::from_min_max(egui::pos2(10.0, 10.0), egui::pos2(130.0, 42.0));
    assert!(lines.iter().all(|(clip, _, _)| *clip == expected_clip));
    assert!(DOUBLE_INNER_OFFSET > DOUBLE_STROKE_WIDTH);
}

#[test]
fn actual_paint_rejects_cache_shape_mismatch() {
    let context = crate::test_ui::Context::default();
    let grid = super::super::painter_tests::grid_surface();
    let output = context.run_ui(egui::RawInput::default(), |ui| {
        let error = paint_prepared(
            ui,
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(VIEWPORT_WIDTH, 100.0)),
            &grid,
            &PreparedGridBorders::default(),
        )
        .expect_err("missing cache must be visible");
        assert!(error.to_string().contains("count"));
    });
    assert!(output.shapes.is_empty());
}

#[test]
fn every_style_emits_expected_width_and_pattern_start() {
    let expected = [
        ("hair", 1.0, 120.0),
        ("thin", 1.0, 120.0),
        ("medium", 2.0, 120.0),
        ("thick", 3.0, 120.0),
        ("double", 1.0, 120.0),
        ("dotted", 1.0, 1.0),
        ("dashed", 1.0, 6.0),
        ("dashDot", 1.0, 6.0),
        ("dashDotDot", 1.0, 6.0),
        ("mediumDashed", 2.0, 6.0),
        ("mediumDashDot", 2.0, 6.0),
        ("mediumDashDotDot", 2.0, 6.0),
        ("slantDashDot", 2.0, 37.0_f32.sqrt()),
        ("solid", 1.0, 120.0),
    ];
    for (style, width, first_length) in expected {
        let lines = painted_lines(style);
        assert!(!lines.is_empty(), "style {style}");
        assert_eq!(lines[0].1.width, width, "style {style}");
        let actual_length = (lines[0].0[1] - lines[0].0[0]).length();
        assert!((actual_length - first_length).abs() < 0.01, "style {style}");
    }
}

#[test]
fn zero_length_and_offscreen_segments_emit_nothing() {
    let context = crate::test_ui::Context::default();
    let zero_length = context.run_ui(egui::RawInput::default(), |ui| {
        let painter = ui.painter();
        emit_pattern(
            painter,
            egui::pos2(4.0, 4.0),
            egui::pos2(4.0, 4.0),
            &[6.0, 3.0],
            egui::Stroke::new(1.0, egui::Color32::default()),
            false,
        );
    });
    assert!(zero_length.shapes.is_empty());
    let grid = super::super::painter_tests::grid_surface();
    let prepared = PreparedGridBorders {
        cells: vec![PreparedCellBorders {
            coordinate: DocumentGridCoordinate { row: 0, column: 0 },
            left: Some(side("solid", "#112233")),
            right: None,
            top: None,
            bottom: None,
        }],
    };
    let offscreen = crate::test_ui::Context::default().run_ui(egui::RawInput::default(), |ui| {
        paint_prepared(
            ui,
            egui::Rect::from_min_size(egui::pos2(400.0, 400.0), egui::vec2(10.0, 10.0)),
            &grid,
            &prepared,
        )
        .expect("offscreen paint");
    });
    assert!(offscreen.shapes.is_empty());
}

#[test]
fn coordinate_mismatch_is_rejected_before_paint() {
    let context = crate::test_ui::Context::default();
    let grid = super::super::painter_tests::grid_surface();
    let prepared = PreparedGridBorders {
        cells: vec![PreparedCellBorders {
            coordinate: DocumentGridCoordinate { row: 1, column: 1 },
            left: None,
            right: None,
            top: None,
            bottom: None,
        }],
    };
    context.run_ui(egui::RawInput::default(), |ui| {
        assert!(paint_prepared(ui, egui::Rect::EVERYTHING, &grid, &prepared).is_err());
    });
}
