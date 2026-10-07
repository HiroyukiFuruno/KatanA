use super::{SpreadsheetFilterUiState, selected_filter_values};
use eframe::egui;
use katana_document_viewer::{
    SpreadsheetAutoFilterArtifact, SpreadsheetCoordinate, SpreadsheetFilterColumnArtifact,
    SpreadsheetFilterCriterion, SpreadsheetFilterEvent, SpreadsheetFilterRange,
    SpreadsheetFrameMetadata,
};

fn metadata(
    sheet_index: usize,
    criteria: Vec<SpreadsheetFilterCriterion>,
) -> SpreadsheetFrameMetadata {
    metadata_for_column(sheet_index, 1, criteria)
}

fn metadata_for_column(
    sheet_index: usize,
    column: usize,
    criteria: Vec<SpreadsheetFilterCriterion>,
) -> SpreadsheetFrameMetadata {
    SpreadsheetFrameMetadata {
        sheet_index,
        visible_row_count: 2,
        auto_filter: Some(SpreadsheetAutoFilterArtifact {
            range: SpreadsheetFilterRange {
                start: SpreadsheetCoordinate::new(0, 0),
                end: SpreadsheetCoordinate::new(2, 2),
            },
            columns: vec![SpreadsheetFilterColumnArtifact {
                column,
                criteria,
                candidates: vec!["North".into(), "South".into()],
            }],
            filtered_out_rows: Vec::new(),
            diagnostics: Vec::new(),
        }),
    }
}

#[test]
fn metadata_projects_values_and_special_criteria() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(
        Some(metadata(
            0,
            vec![
                SpreadsheetFilterCriterion::Values(vec!["North".into()]),
                SpreadsheetFilterCriterion::Blank,
                SpreadsheetFilterCriterion::NonBlank,
            ],
        )),
        None,
    );
    state.request(0, 1);
    assert!(state.selected_values.contains("North"));
    assert!(state.include_blank);
    assert!(state.include_non_blank);
}

#[test]
fn non_blank_criterion_selects_nonempty_candidates_without_empty_value() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(
        Some(metadata(0, vec![SpreadsheetFilterCriterion::NonBlank])),
        None,
    );
    state.request(0, 1);
    state.update_metadata(
        Some(metadata(0, vec![SpreadsheetFilterCriterion::NonBlank])),
        Some(&SpreadsheetFilterEvent::Candidates {
            sheet_index: 0,
            column: 1,
            values: vec![String::new(), "North".into(), "South".into()],
            truncated: false,
        }),
    );
    assert_eq!(
        selected_filter_values(&state),
        vec!["North".to_owned(), "South".to_owned()]
    );
    assert!(!state.include_blank);
}

#[test]
fn stale_candidate_event_is_ignored_after_sheet_change() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(Some(metadata(0, Vec::new())), None);
    state.request(0, 1);
    state.update_metadata(Some(metadata(1, Vec::new())), None);
    state.update_metadata(
        Some(metadata(1, Vec::new())),
        Some(&SpreadsheetFilterEvent::Candidates {
            sheet_index: 0,
            column: 1,
            values: vec!["stale".into()],
            truncated: false,
        }),
    );
    assert!(state.requested.is_none());
    assert!(state.candidate_values.is_empty());
}

#[test]
fn unfiltered_candidate_response_selects_all_values() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(Some(metadata(0, Vec::new())), None);
    state.request(0, 1);
    state.update_metadata(
        Some(metadata(0, Vec::new())),
        Some(&SpreadsheetFilterEvent::Candidates {
            sheet_index: 0,
            column: 1,
            values: vec!["North".into(), "South".into()],
            truncated: false,
        }),
    );
    assert_eq!(
        state.selected_values,
        ["North".to_owned(), "South".to_owned()]
            .into_iter()
            .collect()
    );
    assert!(!state.candidate_truncated);
}

#[test]
fn blank_candidate_is_projected_to_blank_checkbox_once() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(Some(metadata(0, Vec::new())), None);
    state.request(0, 1);
    state.update_metadata(
        Some(metadata(0, Vec::new())),
        Some(&SpreadsheetFilterEvent::Candidates {
            sheet_index: 0,
            column: 1,
            values: vec![String::new(), "North".into()],
            truncated: false,
        }),
    );
    assert!(state.include_blank);
    assert_eq!(state.candidate_values, vec!["North".to_owned()]);
    assert_eq!(
        selected_filter_values(&state),
        vec!["North".to_owned(), String::new()]
    );
}

#[test]
fn clearing_blank_checkbox_excludes_empty_apply_value() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(
        Some(metadata(0, vec![SpreadsheetFilterCriterion::Blank])),
        None,
    );
    state.request(0, 1);
    state.include_blank = false;
    assert!(selected_filter_values(&state).is_empty());
}

#[test]
fn blank_criterion_projects_to_empty_apply_value() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(
        Some(metadata(0, vec![SpreadsheetFilterCriterion::Blank])),
        None,
    );
    state.request(0, 1);
    assert_eq!(selected_filter_values(&state), vec![String::new()]);
}

#[test]
fn truncated_candidate_response_is_retained_as_unsafe_to_apply() {
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(Some(metadata(0, Vec::new())), None);
    state.request(0, 1);
    state.update_metadata(
        Some(metadata(0, Vec::new())),
        Some(&SpreadsheetFilterEvent::Candidates {
            sheet_index: 0,
            column: 1,
            values: vec!["North".into()],
            truncated: true,
        }),
    );
    assert!(state.candidate_truncated);
}

fn pointer_input(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn header_input_requests_candidates_for_an_eligible_filter_column() {
    let context = crate::test_ui::Context::default();
    let mut state = SpreadsheetFilterUiState::default();
    state.update_metadata(Some(metadata(0, Vec::new())), None);
    let grid = super::super::painter_tests::grid_surface();
    let pointer = egui::pos2(120.0, 20.0);
    let input = |events| egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(300.0, 200.0),
        )),
        events,
        ..Default::default()
    };

    for events in [
        vec![egui::Event::PointerMoved(pointer)],
        vec![pointer_input(pointer, true)],
        vec![pointer_input(pointer, false)],
    ] {
        let mut commands = Vec::new();
        context.run_ui(input(events), |ui| {
            commands = super::show(
                ui,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300.0, 200.0)),
                &grid,
                &mut state,
            );
        });
        if commands.iter().any(|command| {
            matches!(
                command,
                katana_document_viewer::SpreadsheetFilterCommand::Candidates {
                    sheet_index: 0,
                    column: 0,
                    limit: 512,
                }
            )
        }) {
            return;
        }
    }

    panic!("eligible header click did not request candidates");
}

#[path = "spreadsheet_filter_controls_ui_tests.rs"]
mod ui_tests;

#[path = "spreadsheet_filter_controls_scroll_tests.rs"]
mod scroll_tests;
