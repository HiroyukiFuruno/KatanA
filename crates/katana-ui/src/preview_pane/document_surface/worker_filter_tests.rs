use super::super::source::DocumentSurfaceSource;
use super::super::worker::{self, DocumentWorkerCommand, DocumentWorkerEvent};
use katana_document_viewer::{SpreadsheetFilterCommand, SpreadsheetFilterEvent};
use std::sync::mpsc::{Receiver, Sender};

const FILTER_WORKER_GENERATION: u64 = 42;
const FRAME_RECEIVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

fn representative_filter_source() -> DocumentSurfaceSource {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../scripts/screenshot/fixtures/v0-22-42-spreadsheet-filter/representative-filter.xlsx",
    );
    DocumentSurfaceSource::local(&fixture).expect("representative filtered XLSX source")
}

fn receive_frame(
    events: &Receiver<DocumentWorkerEvent>,
) -> (
    Box<katana_document_viewer::DocumentFrame>,
    Option<katana_document_viewer::SpreadsheetFrameMetadata>,
    Option<SpreadsheetFilterEvent>,
    usize,
) {
    match events
        .recv_timeout(FRAME_RECEIVE_TIMEOUT)
        .expect("worker frame")
    {
        DocumentWorkerEvent::Frame {
            frame,
            border_cache,
            spreadsheet_metadata,
            filter_event,
            ..
        } => (
            frame,
            spreadsheet_metadata,
            filter_event,
            border_cache.cells.len(),
        ),
        DocumentWorkerEvent::Failure { failure, .. } => panic!("worker failed: {failure}"),
    }
}

fn start_worker() -> (
    Sender<DocumentWorkerCommand>,
    Receiver<DocumentWorkerEvent>,
    std::thread::JoinHandle<()>,
) {
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        worker::run(
            FILTER_WORKER_GENERATION,
            representative_filter_source(),
            command_rx,
            event_tx,
            eframe::egui::Context::default(),
        );
    });
    (command_tx, event_rx, worker)
}

#[test]
fn worker_forwards_xlsx_filter_candidates_apply_and_clear() {
    let (commands, events, worker) = start_worker();

    let (frame, spreadsheet_metadata, filter_event, border_count) = receive_frame(&events);
    let metadata = spreadsheet_metadata.expect("initial spreadsheet metadata");
    assert_eq!(metadata.sheet_index, 0);
    assert!(metadata.auto_filter.is_some());
    assert_eq!(filter_event, None);
    assert_eq!(
        border_count,
        frame.surface.grid().expect("grid").cells.len()
    );

    commands
        .send(DocumentWorkerCommand::SpreadsheetFilter(
            SpreadsheetFilterCommand::Candidates {
                sheet_index: 0,
                column: 0,
                limit: 16,
            },
        ))
        .expect("candidate command");
    let (_, spreadsheet_metadata, filter_event, _) = receive_frame(&events);
    let SpreadsheetFilterEvent::Candidates { values, .. } = filter_event.expect("candidate event")
    else {
        panic!("expected candidate event");
    };
    assert!(values.iter().any(|value| value == "North"));
    assert!(
        spreadsheet_metadata
            .and_then(|metadata| metadata.auto_filter)
            .is_some()
    );

    commands
        .send(DocumentWorkerCommand::SpreadsheetFilter(
            SpreadsheetFilterCommand::ApplyValues {
                sheet_index: 0,
                column: 0,
                values: vec!["North".to_owned()],
            },
        ))
        .expect("apply command");
    let (_, spreadsheet_metadata, filter_event, _) = receive_frame(&events);
    assert!(matches!(
        filter_event,
        Some(SpreadsheetFilterEvent::VisibilityChanged {
            visible_row_count: 4,
            ..
        })
    ));
    assert_eq!(
        spreadsheet_metadata
            .expect("filtered spreadsheet metadata")
            .visible_row_count,
        4
    );

    commands
        .send(DocumentWorkerCommand::SpreadsheetFilter(
            SpreadsheetFilterCommand::Clear {
                sheet_index: 0,
                column: None,
            },
        ))
        .expect("clear command");
    let (_, spreadsheet_metadata, filter_event, _) = receive_frame(&events);
    assert!(matches!(
        filter_event,
        Some(SpreadsheetFilterEvent::VisibilityChanged {
            visible_row_count: 7,
            ..
        })
    ));
    assert_eq!(
        spreadsheet_metadata
            .expect("cleared spreadsheet metadata")
            .visible_row_count,
        7
    );

    drop(commands);
    worker.join().expect("worker thread");
}
