use super::worker::DocumentWorkerCommand;
use super::worker_tests::idle_surface;
use katana_document_viewer::{DocumentViewerCommand, SpreadsheetFilterCommand};

fn queued_filter_mutation(index: usize) -> DocumentWorkerCommand {
    DocumentWorkerCommand::SpreadsheetFilter(SpreadsheetFilterCommand::ApplyValues {
        sheet_index: 0,
        column: 2,
        values: vec![if index % 2 == 0 {
            "North".to_owned()
        } else {
            "South".to_owned()
        }],
    })
}

#[test]
fn queue_rejection_preserves_inflight_work_and_existing_pending_commands() {
    let (mut surface, _command_rx, _event_tx) = idle_surface();
    surface.command_in_flight = true;
    for index in 0..16 {
        assert!(surface.pending_commands.push(queued_filter_mutation(index)));
    }

    surface.queue(DocumentWorkerCommand::SpreadsheetFilter(
        SpreadsheetFilterCommand::Clear {
            sheet_index: 0,
            column: None,
        },
    ));

    assert_eq!(
        surface.failure.as_ref().map(|failure| failure.operation),
        Some("enqueue")
    );
    assert!(surface.command_in_flight);
    assert_eq!(
        surface.pending_commands.take_next(),
        Some(queued_filter_mutation(0))
    );
}

#[test]
fn full_channel_rejection_preserves_existing_pending_commands() {
    let (mut surface, _initial_command_rx, _event_tx) = idle_surface();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    sender
        .send(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next))
        .expect("prefill command channel");
    surface.command_tx = Some(sender);
    for index in 0..16 {
        assert!(surface.pending_commands.push(queued_filter_mutation(index)));
    }

    surface.send(DocumentWorkerCommand::SpreadsheetFilter(
        SpreadsheetFilterCommand::Clear {
            sheet_index: 0,
            column: None,
        },
    ));

    assert_eq!(
        surface.failure.as_ref().map(|failure| failure.operation),
        Some("enqueue")
    );
    assert!(!surface.command_in_flight);
    assert_eq!(
        surface.pending_commands.take_next(),
        Some(queued_filter_mutation(0))
    );
    drop(receiver);
}
