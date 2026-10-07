use eframe::egui;
use egui_kittest::Harness;
use katana_document_viewer::SpreadsheetFilterCommand;

use super::{
    SharedFilterCommands, SharedFilterState, TEST_COLUMN, TEST_SHEET, click_clear,
    deliver_candidates, menu_labels, populated_filter_state, run_filter_harness_with_width,
    take_commands,
};

#[test]
fn narrow_header_real_input_requests_candidates_and_clears_existing_filter() {
    const COLUMN_WIDTHS: [u32; 4] = [1, 12, 22, 23];
    for width in COLUMN_WIDTHS {
        verify_narrow_header_filter_input(width);
    }
}

#[test]
fn zero_width_header_has_no_filter_hit_target() {
    let mut cell = super::super::super::super::painter_tests::grid_cell();
    cell.bounds.width = 0;
    cell.clipped_bounds.width = 0;
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, super::TEST_VIEWPORT);
    assert!(super::super::super::filter_button_rect(viewport, &cell).is_none());
}

fn verify_narrow_header_filter_input(width: u32) {
    let (mut harness, state, commands) =
        run_filter_harness_with_width(populated_filter_state(false), Some(width));
    let bounds = super::super::super::super::painter_tests::grid_cell().bounds;
    let pointer = egui::pos2(bounds.x as f32 + width as f32 / 2.0, bounds.y as f32 + 1.0);
    harness.event(egui::Event::PointerMoved(pointer));
    for pressed in [true, false] {
        harness.event(super::super::pointer_input(pointer, pressed));
        harness.step();
    }
    assert!(
        matches!(
            take_commands(&commands).as_slice(),
            [SpreadsheetFilterCommand::Candidates {
                sheet_index: TEST_SHEET,
                column: TEST_COLUMN,
                limit: 512,
            }]
        ),
        "narrow column {width} must retain its filter hit target"
    );
    verify_narrow_header_clear(&mut harness, &state, &commands);
}

fn verify_narrow_header_clear(
    harness: &mut Harness<'static>,
    state: &SharedFilterState,
    commands: &SharedFilterCommands,
) {
    deliver_candidates(state, false);
    harness.run();
    let (_, _, clear_label) = menu_labels();
    click_clear(harness, state, commands, &clear_label);
    harness.run();
    assert_eq!(
        take_commands(commands),
        vec![SpreadsheetFilterCommand::Clear {
            sheet_index: TEST_SHEET,
            column: Some(TEST_COLUMN),
        }]
    );
}
