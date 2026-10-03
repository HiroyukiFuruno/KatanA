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
