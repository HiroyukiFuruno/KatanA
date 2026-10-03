use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use katana_document_viewer::{
    SpreadsheetFilterCommand, SpreadsheetFilterCriterion, SpreadsheetFilterEvent,
};

use super::{SpreadsheetFilterUiState, metadata_for_column};

const TEST_VIEWPORT: egui::Vec2 = egui::vec2(320.0, 240.0);
const TEST_COLUMN: usize = 0;
const TEST_SHEET: usize = 0;

type SharedFilterState = Arc<Mutex<SpreadsheetFilterUiState>>;
type SharedFilterCommands = Arc<Mutex<Vec<SpreadsheetFilterCommand>>>;

fn run_filter_harness(
    state: SpreadsheetFilterUiState,
) -> (Harness<'static>, SharedFilterState, SharedFilterCommands) {
    let shared_state = Arc::new(Mutex::new(state));
    let shared_commands = Arc::new(Mutex::new(Vec::new()));
    let rendered_state = Arc::clone(&shared_state);
    let rendered_commands = Arc::clone(&shared_commands);
    let grid = super::super::super::painter_tests::grid_surface();
    let mut harness = Harness::builder()
        .with_size(TEST_VIEWPORT)
        .build_ui(move |ui| {
            let mut state = rendered_state.lock().expect("filter state lock");
            let commands = super::super::show(
                ui,
                egui::Rect::from_min_size(egui::Pos2::ZERO, TEST_VIEWPORT),
                &grid,
                &mut state,
            );
            rendered_commands
                .lock()
                .expect("filter command lock")
                .extend(commands);
        });
    harness.run();
    (harness, shared_state, shared_commands)
}

fn populated_filter_state(truncated: bool) -> SpreadsheetFilterUiState {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(
        Some(metadata_for_column(
            TEST_SHEET,
            TEST_COLUMN,
            vec![
                SpreadsheetFilterCriterion::Values(vec!["North".to_owned()]),
                SpreadsheetFilterCriterion::Blank,
            ],
        )),
        None,
    );
    state.candidate_truncated = truncated;
    state
}

fn take_commands(
    commands: &Arc<Mutex<Vec<SpreadsheetFilterCommand>>>,
) -> Vec<SpreadsheetFilterCommand> {
    std::mem::take(&mut *commands.lock().expect("filter command lock"))
}

fn menu_labels() -> (String, String, String) {
    let messages = &crate::i18n::I18nOps::get().preview.spreadsheet_filter;
    (
        format!("{} {}", messages.open, TEST_COLUMN + 1),
        messages.apply.clone(),
        messages.clear.clone(),
    )
}

fn deliver_candidates(state: &Arc<Mutex<SpreadsheetFilterUiState>>, truncated: bool) {
    let event = SpreadsheetFilterEvent::Candidates {
        sheet_index: TEST_SHEET,
        column: TEST_COLUMN,
        values: vec!["North".to_owned(), "South".to_owned(), String::new()],
        truncated,
    };
    let mut state = state.lock().expect("filter state lock");
    assert_eq!(state.requested, Some((TEST_SHEET, TEST_COLUMN)));
    state.update_metadata(
        Some(metadata_for_column(
            TEST_SHEET,
            TEST_COLUMN,
            vec![
                SpreadsheetFilterCriterion::Values(vec!["North".to_owned()]),
                SpreadsheetFilterCriterion::Blank,
            ],
        )),
        Some(&event),
    );
}

fn click_clear(
    harness: &mut Harness<'static>,
    state: &Arc<Mutex<SpreadsheetFilterUiState>>,
    commands: &Arc<Mutex<Vec<SpreadsheetFilterCommand>>>,
    label: &str,
) {
    let node = harness.root().query_by_label(label);
    if node.is_none() {
        eprintln!(
            "clear label missing: root_bounds={:?}, state={:?}, commands={:?}",
            harness.root().accesskit_node().raw_bounds(),
            state.lock().expect("filter state lock"),
            commands.lock().expect("filter command lock"),
        );
    }
    node.expect("clear control must remain available").click();
}

#[test]
fn real_menu_apply_dispatches_selected_values_and_clear_dispatches_clear() {
    let (mut harness, state, commands) = run_filter_harness(populated_filter_state(false));
    let (open_label, apply_label, clear_label) = menu_labels();
    harness.get_by_label(&open_label).click();
    harness.run();
    assert!(matches!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::Candidates {
            sheet_index: TEST_SHEET,
            column: TEST_COLUMN,
            limit: 512,
        }]
    ));
    deliver_candidates(&state, false);
    harness.run();
    harness.get_by_label(&apply_label).click();
    harness.run();
    assert_eq!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::ApplyValues {
            sheet_index: TEST_SHEET,
            column: TEST_COLUMN,
            values: vec!["North".to_owned(), String::new()],
        }]
    );

    harness.get_by_label(&open_label).click();
    harness.run();
    assert!(matches!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::Candidates {
            sheet_index: TEST_SHEET,
            column: TEST_COLUMN,
            limit: 512,
        }]
    ));
    deliver_candidates(&state, false);
    harness.run();
    click_clear(&mut harness, &state, &commands, &clear_label);
    harness.run();
    assert_eq!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::Clear {
            sheet_index: TEST_SHEET,
            column: Some(TEST_COLUMN),
        }]
    );
}

#[test]
fn real_menu_disables_apply_but_keeps_clear_for_truncated_candidates() {
    let (mut harness, state, commands) = run_filter_harness(populated_filter_state(true));
    let (open_label, apply_label, clear_label) = menu_labels();
    harness.get_by_label(&open_label).click();
    harness.run();
    assert!(matches!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::Candidates {
            sheet_index: TEST_SHEET,
            column: TEST_COLUMN,
            limit: 512,
        }]
    ));
    deliver_candidates(&state, true);
    harness.run();
    let apply = harness.get_by_label(&apply_label);
    for control in [&apply, &harness.get_by_label(&clear_label)] {
        let bounds = control
            .accesskit_node()
            .raw_bounds()
            .expect("control bounds");
        assert!(bounds.y0 >= 0.0 && bounds.y1 <= f64::from(TEST_VIEWPORT.y));
        assert!(bounds.x0 >= 0.0 && bounds.x1 <= f64::from(TEST_VIEWPORT.x));
    }
    apply.click();
    harness.run();
    assert!(take_commands(&commands).is_empty());

    click_clear(&mut harness, &state, &commands, &clear_label);
    harness.run();
    assert_eq!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::Clear {
            sheet_index: TEST_SHEET,
            column: Some(TEST_COLUMN),
        }]
    );
}

#[test]
fn real_menu_select_all_keeps_the_menu_open_until_apply() {
    let (mut harness, state, commands) = run_filter_harness(populated_filter_state(false));
    let (open_label, apply_label, _) = menu_labels();
    let select_all = crate::i18n::I18nOps::get()
        .preview
        .spreadsheet_filter
        .select_all
        .clone();
    harness.get_by_label(&open_label).click();
    harness.run();
    assert!(matches!(
        take_commands(&commands).as_slice(),
        [SpreadsheetFilterCommand::Candidates { .. }]
    ));
    deliver_candidates(&state, false);
    harness.run();
    harness.get_by_label(&select_all).click();
    harness.run();
    assert!(take_commands(&commands).is_empty());
    harness.get_by_label(&apply_label).click();
    harness.run();
    assert_eq!(
        take_commands(&commands),
        vec![SpreadsheetFilterCommand::ApplyValues {
            sheet_index: TEST_SHEET,
            column: TEST_COLUMN,
            values: vec!["North".to_owned(), "South".to_owned(), String::new()],
        }]
    );
}

#[path = "spreadsheet_filter_controls_unsupported_ui_tests.rs"]
mod unsupported_ui_tests;
