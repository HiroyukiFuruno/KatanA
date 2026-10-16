use super::{MAX_PENDING_USER_COMMANDS, PendingDocumentCommands};
use crate::preview_pane::document_surface::worker::DocumentWorkerCommand;
use katana_document_viewer::{
    DocumentGridCommand, DocumentSurfaceCommand, DocumentViewerCommand, SpreadsheetFilterCommand,
};

const CANDIDATE_LIMIT: usize = 16;

fn candidates(sheet_index: usize, column: usize) -> DocumentWorkerCommand {
    DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::Candidates {
        sheet_index,
        column,
        limit: CANDIDATE_LIMIT,
    })
}

fn next() -> DocumentWorkerCommand {
    DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next)
}

fn clear() -> DocumentWorkerCommand {
    DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::Clear {
        sheet_index: 0,
        column: None,
    })
}

fn apply(values: &[&str]) -> DocumentWorkerCommand {
    DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::ApplyValues {
        sheet_index: 0,
        column: 2,
        values: values.iter().map(|value| (*value).to_owned()).collect(),
    })
}

#[test]
fn saturated_navigation_preserves_candidate_and_sixteen_latest_commands() {
    let mut pending = PendingDocumentCommands::default();
    let candidate = candidates(0, 2);
    pending.push(candidate.clone());
    for _ in 0..32 {
        pending.push(next());
    }
    assert_eq!(pending.take_next(), Some(candidate));
    for _ in 0..MAX_PENDING_USER_COMMANDS {
        assert_eq!(pending.take_next(), Some(next()));
    }
    assert!(pending.is_empty());
}

#[test]
fn latest_candidate_supersedes_only_pending_request_at_its_new_position() {
    let mut pending = PendingDocumentCommands::default();
    pending.push(next());
    pending.push(candidates(0, 2));
    pending.push(next());
    let latest = candidates(1, 3);
    pending.push(latest.clone());
    pending.push(next());
    assert_eq!(pending.take_next(), Some(next()));
    assert_eq!(pending.take_next(), Some(next()));
    assert_eq!(pending.take_next(), Some(latest));
    assert_eq!(pending.take_next(), Some(next()));
    assert!(pending.is_empty());
}

#[test]
fn candidate_preservation_keeps_clear_apply_fifo_order() {
    let mut pending = PendingDocumentCommands::default();
    let clear = DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::Clear {
        sheet_index: 0,
        column: None,
    });
    let apply = DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::ApplyValues {
        sheet_index: 0,
        column: 2,
        values: vec!["North".to_owned()],
    });
    for _ in 0..MAX_PENDING_USER_COMMANDS {
        pending.push(next());
    }
    pending.push(clear.clone());
    pending.push(candidates(0, 2));
    pending.push(apply.clone());
    for _ in 0..MAX_PENDING_USER_COMMANDS - 2 {
        assert_eq!(pending.take_next(), Some(next()));
    }
    assert_eq!(pending.take_next(), Some(clear));
    assert_eq!(pending.take_next(), Some(candidates(0, 2)));
    assert_eq!(pending.take_next(), Some(apply));
    assert!(pending.is_empty());
}

#[test]
fn repeated_candidate_requests_remain_bounded_and_clearable() {
    let mut pending = PendingDocumentCommands::default();
    for column in 0..32 {
        pending.push(candidates(0, column));
        pending.push(next());
    }
    assert_eq!(pending.user.len(), MAX_PENDING_USER_COMMANDS + 1);
    pending.clear();
    assert!(pending.is_empty());
    assert_eq!(pending.take_next(), None);
}

#[test]
fn candidate_keeps_coalescing_on_each_side_of_its_ordering_barrier() {
    let mut pending = PendingDocumentCommands::default();
    let scroll = |x| {
        DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(
            DocumentGridCommand::ScrollTo { x, y: 0 },
        ))
    };
    pending.push(scroll(1));
    pending.push(scroll(2));
    pending.push(candidates(0, 2));
    pending.push(scroll(3));
    pending.push(scroll(4));
    assert_eq!(pending.take_next(), Some(scroll(2)));
    assert_eq!(pending.take_next(), Some(candidates(0, 2)));
    assert_eq!(pending.take_next(), Some(scroll(4)));
    assert!(pending.is_empty());
}

#[test]
fn saturated_navigation_preserves_clear_and_apply_fifo_order() {
    let mut pending = PendingDocumentCommands::default();
    let clear = clear();
    let candidate = candidates(0, 2);
    let apply = apply(&["North"]);
    for _ in 0..MAX_PENDING_USER_COMMANDS {
        pending.push(next());
    }
    pending.push(clear.clone());
    pending.push(candidate.clone());
    pending.push(apply.clone());
    for _ in 0..MAX_PENDING_USER_COMMANDS {
        pending.push(next());
    }

    let drained = std::iter::from_fn(|| pending.take_next()).collect::<Vec<_>>();
    let clear_index = drained.iter().position(|command| *command == clear);
    let candidate_index = drained.iter().position(|command| *command == candidate);
    let apply_index = drained.iter().position(|command| *command == apply);
    assert!(clear_index.is_some());
    assert!(candidate_index.is_some());
    assert!(apply_index.is_some());
    assert!(clear_index < candidate_index);
    assert!(candidate_index < apply_index);
}

#[test]
fn full_critical_queue_rejects_new_navigation_without_dropping_critical_commands() {
    let mut pending = PendingDocumentCommands::default();
    let critical = (0..MAX_PENDING_USER_COMMANDS)
        .map(|index| apply(&[if index % 2 == 0 { "North" } else { "South" }]))
        .collect::<Vec<_>>();
    for command in &critical {
        pending.push(command.clone());
    }
    assert!(!pending.push(next()));

    let drained = std::iter::from_fn(|| pending.take_next()).collect::<Vec<_>>();
    assert_eq!(drained, critical);
}

#[test]
fn full_critical_queue_rejects_new_filter_mutation_without_dropping_existing_commands() {
    let mut pending = PendingDocumentCommands::default();
    let critical = (0..MAX_PENDING_USER_COMMANDS)
        .map(|index| apply(&[if index % 2 == 0 { "North" } else { "South" }]))
        .collect::<Vec<_>>();
    for command in &critical {
        pending.push(command.clone());
    }
    assert!(!pending.push(clear()));

    let drained = std::iter::from_fn(|| pending.take_next()).collect::<Vec<_>>();
    assert_eq!(drained, critical);
}
