use eframe::egui;
use katana_document_viewer::{
    DocumentGridBorderSide, DocumentGridCellBorders, DocumentGridCoordinate,
};

use super::{BorderPreparationError, PreparedBorderStyle, prepare_entries};

const STYLES: [(&str, PreparedBorderStyle); 14] = [
    ("hair", PreparedBorderStyle::Hair),
    ("thin", PreparedBorderStyle::Thin),
    ("medium", PreparedBorderStyle::Medium),
    ("thick", PreparedBorderStyle::Thick),
    ("double", PreparedBorderStyle::Double),
    ("dotted", PreparedBorderStyle::Dotted),
    ("dashed", PreparedBorderStyle::Dashed),
    ("dashDot", PreparedBorderStyle::DashDot),
    ("dashDotDot", PreparedBorderStyle::DashDotDot),
    ("mediumDashed", PreparedBorderStyle::MediumDashed),
    ("mediumDashDot", PreparedBorderStyle::MediumDashDot),
    ("mediumDashDotDot", PreparedBorderStyle::MediumDashDotDot),
    ("slantDashDot", PreparedBorderStyle::SlantDashDot),
    ("solid", PreparedBorderStyle::Solid),
];
const COLORS: [&str; 4] = ["#112233", "#445566", "#778899", "#AABBCC"];
const LARGE_BATCH_ENTRIES: usize = 4096;
const LARGE_BATCH_COLUMNS: usize = 64;
const PREFIX_ENTRIES: usize = 128;
const PREFIX_COLUMNS: usize = 16;
const BOTTOM_SIDE_INDEX: usize = 3;

#[test]
fn large_mixed_batch_preserves_order_coordinates_and_all_sides() {
    let entries = large_mixed_entries();
    let prepared = prepare_entries(&entries).expect("valid mixed batch");
    assert_eq!(4096, prepared.cells.len());
    assert_projected_entries(&entries, &prepared.cells);
}

fn large_mixed_entries() -> Vec<(DocumentGridCoordinate, DocumentGridCellBorders)> {
    (0..LARGE_BATCH_ENTRIES)
        .map(|index| {
            let coordinate = coordinate(index / LARGE_BATCH_COLUMNS, index % LARGE_BATCH_COLUMNS);
            let borders = if index.is_multiple_of(2) {
                visible_borders(index)
            } else {
                DocumentGridCellBorders::default()
            };
            (coordinate, borders)
        })
        .collect()
}

fn visible_borders(index: usize) -> DocumentGridCellBorders {
    DocumentGridCellBorders {
        left: Some(source_side(index, 0)),
        right: Some(source_side(index, 1)),
        top: Some(source_side(index, 2)),
        bottom: Some(source_side(index, BOTTOM_SIDE_INDEX)),
    }
}

fn source_side(index: usize, side: usize) -> DocumentGridBorderSide {
    DocumentGridBorderSide {
        style: STYLES[(index + side) % STYLES.len()].0.to_owned(),
        color: Some(COLORS[side].to_owned()),
    }
}

fn assert_projected_entries(
    entries: &[(DocumentGridCoordinate, DocumentGridCellBorders)],
    cells: &[super::PreparedCellBorders],
) {
    for (index, ((coordinate, _), cell)) in entries.iter().zip(cells).enumerate() {
        assert_eq!(*coordinate, cell.coordinate, "entry order at {index}");
        assert_projected_cell(index, cell);
    }
}

fn assert_projected_cell(index: usize, cell: &super::PreparedCellBorders) {
    let sides = [&cell.left, &cell.right, &cell.top, &cell.bottom];
    if !index.is_multiple_of(2) {
        assert!(sides.into_iter().all(Option::is_none));
        return;
    }
    for (side_index, side) in sides.into_iter().enumerate() {
        let side = side.as_ref().expect("visible side retained");
        assert_eq!(STYLES[(index + side_index) % STYLES.len()].1, side.style);
        let expected_color = egui::Color32::from_hex(COLORS[side_index]).unwrap();
        assert_eq!(Some(expected_color), side.color);
    }
}

#[test]
fn empty_batch_is_accepted() {
    let prepared = prepare_entries(&[]).expect("empty batch is valid");
    assert!(prepared.cells.is_empty());
}

#[test]
fn late_unknown_style_reports_coordinate_and_side() {
    let at = coordinate(8, 9);
    let entries = with_late_side(
        at,
        "bottom",
        DocumentGridBorderSide {
            style: "not-a-border".to_owned(),
            color: None,
        },
    );
    assert_eq!(
        Err(BorderPreparationError::UnknownStyle {
            coordinate: at,
            side: "bottom",
            style: "not-a-border".to_owned(),
        }),
        prepare_entries(&entries)
    );
}

#[test]
fn late_invalid_color_reports_coordinate_and_side() {
    let at = coordinate(8, 9);
    let entries = with_late_side(
        at,
        "top",
        DocumentGridBorderSide {
            style: "thin".to_owned(),
            color: Some("not-a-color".to_owned()),
        },
    );
    assert_eq!(
        Err(BorderPreparationError::InvalidColor {
            coordinate: at,
            side: "top",
            color: "not-a-color".to_owned(),
        }),
        prepare_entries(&entries)
    );
}

fn with_late_side(
    at: DocumentGridCoordinate,
    side: &str,
    border: DocumentGridBorderSide,
) -> Vec<(DocumentGridCoordinate, DocumentGridCellBorders)> {
    let mut entries = (0..PREFIX_ENTRIES)
        .map(|index| {
            (
                coordinate(index / PREFIX_COLUMNS, index % PREFIX_COLUMNS),
                DocumentGridCellBorders::default(),
            )
        })
        .collect::<Vec<_>>();
    let mut borders = DocumentGridCellBorders::default();
    match side {
        "bottom" => borders.bottom = Some(border),
        "top" => borders.top = Some(border),
        _ => unreachable!("test uses a known side"),
    }
    entries.push((at, borders));
    entries
}

fn coordinate(row: usize, column: usize) -> DocumentGridCoordinate {
    DocumentGridCoordinate { row, column }
}
