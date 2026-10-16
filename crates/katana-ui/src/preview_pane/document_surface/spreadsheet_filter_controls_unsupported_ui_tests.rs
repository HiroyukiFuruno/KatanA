use egui_kittest::kittest::{NodeT, Queryable};
use katana_document_viewer::{SpreadsheetFilterCriterion, SpreadsheetFilterEvent};

use super::{TEST_COLUMN, TEST_SHEET, click_clear, menu_labels, run_filter_harness, take_commands};

#[test]
fn unsupported_filter_disables_apply_but_allows_explicit_clear() {
    let (mut harness, state, commands) = run_filter_harness(unsupported_state());
    let (open_label, apply_label, clear_label) = menu_labels();
    open_filter_menu(&mut harness, &open_label, &commands);
    deliver_candidates(&state);
    harness.run();
    assert_unsupported_apply_blocked(&mut harness, &state, &commands, &apply_label);
    assert_clear_allowed(&mut harness, &state, &commands, &clear_label);
}

fn open_filter_menu(
    harness: &mut egui_kittest::Harness<'static>,
    open_label: &str,
    commands: &super::SharedFilterCommands,
) {
    harness.get_by_label(open_label).click();
    harness.run();
    take_commands(commands);
}

fn assert_unsupported_apply_blocked(
    harness: &mut egui_kittest::Harness<'static>,
    state: &super::SharedFilterState,
    commands: &super::SharedFilterCommands,
    apply_label: &str,
) {
    assert!(
        state
            .lock()
            .expect("filter state lock")
            .has_unsupported_criterion
    );
    let command = super::super::super::apply_command(
        &state.lock().expect("filter state lock"),
        TEST_SHEET,
        TEST_COLUMN,
    );
    assert!(command.is_none());
    let apply = harness.get_by_label(apply_label);
    assert!(apply.accesskit_node().is_disabled());
    apply.click();
    harness.run();
    assert!(take_commands(commands).is_empty());
}

fn assert_clear_allowed(
    harness: &mut egui_kittest::Harness<'static>,
    state: &super::SharedFilterState,
    commands: &super::SharedFilterCommands,
    clear_label: &str,
) {
    click_clear(harness, state, commands, clear_label);
    harness.run();
    assert!(matches!(
        take_commands(commands).as_slice(),
        [katana_document_viewer::SpreadsheetFilterCommand::Clear {
            sheet_index: TEST_SHEET,
            column: Some(TEST_COLUMN),
        }]
    ));
}

fn unsupported_state() -> super::super::SpreadsheetFilterUiState {
    let mut state = super::super::SpreadsheetFilterUiState::default();
    state.update_metadata(
        Some(super::super::metadata_for_column(
            TEST_SHEET,
            TEST_COLUMN,
            vec![SpreadsheetFilterCriterion::Unsupported(
                "customFilters".to_owned(),
            )],
        )),
        None,
    );
    state
}

fn deliver_candidates(state: &super::SharedFilterState) {
    let event = SpreadsheetFilterEvent::Candidates {
        sheet_index: TEST_SHEET,
        column: TEST_COLUMN,
        values: vec!["North".to_owned(), "South".to_owned()],
        truncated: false,
    };
    state.lock().expect("filter state lock").update_metadata(
        Some(super::super::metadata_for_column(
            TEST_SHEET,
            TEST_COLUMN,
            vec![SpreadsheetFilterCriterion::Unsupported(
                "customFilters".to_owned(),
            )],
        )),
        Some(&event),
    );
}
