use super::super::worker::{DocumentWorkerCommand, DocumentWorkerEvent};
use super::super::worker_tests::idle_surface;
use super::super::{source::DocumentSurfaceSource, types::DocumentSurface, worker};
use katana_document_viewer::{
    DocumentGridCommand, DocumentGridNavigation, DocumentSurfaceCommand, DocumentViewport,
    SpreadsheetFilterCommand,
};

const FRAME_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const VIEWPORT_WIDTH: u32 = 800;
const VIEWPORT_HEIGHT: u32 = 600;
const CANDIDATE_LIMIT: usize = 16;
const SATURATING_NAVIGATION_COUNT: usize = 32;

fn filter_surface() -> (DocumentSurface, std::thread::JoinHandle<()>) {
    let (mut surface, commands, events) = idle_surface();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../scripts/screenshot/fixtures/v0-22-42-spreadsheet-filter/representative-filter.xlsx",
    );
    let source = DocumentSurfaceSource::local(&path).expect("filter fixture");
    surface.source = source.descriptor();
    surface.frame = None;
    surface.command_in_flight = true;
    let worker = std::thread::spawn(move || {
        worker::run(
            1,
            source,
            commands,
            events,
            eframe::egui::Context::default(),
        );
    });
    (surface, worker)
}

fn receive(surface: &mut DocumentSurface) -> DocumentWorkerEvent {
    let event = surface
        .event_rx
        .recv_timeout(FRAME_TIMEOUT)
        .expect("real XLSX worker event");
    if let DocumentWorkerEvent::Failure { failure, .. } = &event {
        panic!("real XLSX worker failed: {failure}");
    }
    event
}

fn queue_saturated_request(surface: &mut DocumentSurface) {
    surface.queue_surface(DocumentSurfaceCommand::Resize(DocumentViewport::new(
        VIEWPORT_WIDTH,
        VIEWPORT_HEIGHT,
    )));
    surface.filter_ui.request(0, 0);
    surface.queue(DocumentWorkerCommand::SpreadsheetFilter(
        SpreadsheetFilterCommand::Candidates {
            sheet_index: 0,
            column: 0,
            limit: CANDIDATE_LIMIT,
        },
    ));
    for _ in 0..SATURATING_NAVIGATION_COUNT {
        surface.queue_surface(DocumentSurfaceCommand::Grid(
            DocumentGridCommand::Navigate {
                intent: DocumentGridNavigation::Down,
                extend: false,
            },
        ));
    }
    assert!(surface.filter_ui.loading);
}

#[test]
fn saturated_navigation_delivers_candidates_and_clears_filter_loading() {
    let ctx = eframe::egui::Context::default();
    let (mut surface, worker) = filter_surface();
    let initial = receive(&mut surface);
    surface.apply_event(&ctx, initial);
    queue_saturated_request(&mut surface);
    let resized = receive(&mut surface);
    surface.apply_event(&ctx, resized);
    surface.poll(&ctx);
    let candidate = receive(&mut surface);
    surface.apply_event(&ctx, candidate);
    assert_eq!(surface.filter_ui.requested, Some((0, 0)));
    assert!(!surface.filter_ui.loading);
    assert!(
        surface
            .filter_ui
            .candidate_values
            .iter()
            .any(|value| value == "North")
    );
    surface.command_tx.take();
    worker.join().expect("real XLSX worker closes");
}
