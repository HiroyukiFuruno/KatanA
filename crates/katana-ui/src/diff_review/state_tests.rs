use std::path::PathBuf;

use katana_platform::DiffViewMode;

use super::{DiffReviewDecision, DiffReviewFile, DiffReviewState};

fn file(name: &str) -> DiffReviewFile {
    DiffReviewFile::new(
        PathBuf::from(name),
        "before".to_string(),
        "after".to_string(),
    )
}

#[test]
fn mark_current_advances_to_next_pending_file() {
    let mut state = DiffReviewState::new(
        vec![file("a.md"), file("b.md")],
        DiffViewMode::Split,
        Some(PathBuf::from("a.md")),
    );

    state.mark_current(DiffReviewDecision::Applied);

    assert_eq!(state.current_index, 1);
    assert_eq!(state.files[0].decision, DiffReviewDecision::Applied);
    assert!(!state.is_complete());
}

#[test]
fn complete_when_all_files_are_decided() {
    let mut state = DiffReviewState::new(vec![file("a.md")], DiffViewMode::Inline, None);

    state.mark_current(DiffReviewDecision::Rejected);

    assert!(state.is_complete());
    assert_eq!(state.current_index, 0);
}

#[test]
fn move_previous_and_next_keep_index_inside_file_pages() {
    let mut state =
        DiffReviewState::new(vec![file("a.md"), file("b.md")], DiffViewMode::Split, None);

    state.move_previous();
    assert_eq!(state.current_index, 0);

    state.move_next();
    assert_eq!(state.current_index, 1);

    state.move_next();
    assert_eq!(state.current_index, 1);

    state.move_previous();
    assert_eq!(state.current_index, 0);
}

#[test]
fn mark_current_prefers_next_pending_file_after_current_page() {
    let mut state = DiffReviewState::new(
        vec![file("a.md"), file("b.md"), file("c.md")],
        DiffViewMode::Split,
        None,
    );
    state.move_next();

    state.mark_current(DiffReviewDecision::Applied);

    assert_eq!(state.current_index, 2);
}

#[test]
fn reject_all_pending_marks_every_file_as_rejected() {
    let mut state = DiffReviewState::new(
        vec![file("a.md"), file("b.md"), file("c.md")],
        DiffViewMode::Split,
        None,
    );
    state.mark_current(DiffReviewDecision::Applied);
    state.reject_all_pending();

    let all_rejected = state
        .files
        .iter()
        .all(|file| file.decision == DiffReviewDecision::Rejected);
    assert!(all_rejected);
    assert!(state.is_complete());
}

#[test]
fn display_name_strips_workspace_root() {
    let f = file("workspace/a.md");
    let display = f.display_name(Some(std::path::Path::new("workspace")));
    assert_eq!(display, "a.md");
}

#[test]
fn current_file_display_name_respects_workspace_root() {
    let state = DiffReviewState::new(vec![file("workspace/a.md")], DiffViewMode::Split, None)
        .with_workspace_root(Some(std::path::PathBuf::from("workspace")));

    assert_eq!(state.current_file_display_name(), "a.md");
}

#[test]
fn can_move_previous_and_next_behaviour() {
    let mut state =
        DiffReviewState::new(vec![file("a.md"), file("b.md")], DiffViewMode::Split, None);
    assert!(!state.can_move_previous());
    assert!(state.can_move_next());

    state.move_next();
    assert!(state.can_move_previous());
    assert!(!state.can_move_next());
}
