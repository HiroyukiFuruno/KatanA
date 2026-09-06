use super::source::DocumentSurfaceSource;
use super::worker::{DocumentWorkerCommand, DocumentWorkerEvent};
use katana_document_viewer::{
    DocumentFitMode, DocumentFrame, DocumentSessionEvent, DocumentSurfaceFrame,
    DocumentViewerCommand, DocumentViewerEvent, DocumentViewerState, PdfRenderedPage,
    ViewerCapabilities, ViewerDocumentFormat, ViewerImageSurface,
};

fn representative_pdf_source() -> DocumentSurfaceSource {
    DocumentSurfaceSource::remote(
        "https://example.test/representative.pdf".to_owned(),
        Some("application/pdf"),
        include_bytes!(
            "../../../../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf"
        )
        .to_vec(),
    )
    .expect("representative PDF source")
}

fn test_frame() -> DocumentFrame {
    let surface = DocumentSurfaceFrame::from_rendered_page(
        "test page",
        PdfRenderedPage {
            page_index: 0,
            scale: 1.0,
            surface: ViewerImageSurface {
                fingerprint: "test-page".to_owned(),
                width: 1,
                height: 1,
                display_width: 1.0,
                display_height: 1.0,
                content_scale: 100,
                rgba: vec![0, 0, 0, 255],
            },
        },
    )
    .expect("test document frame");
    DocumentFrame {
        surface,
        state: DocumentViewerState::new(2),
        capabilities: ViewerCapabilities::static_page(),
        diagnostics: Vec::new(),
        format: ViewerDocumentFormat::Pdf,
        spreadsheet: None,
    }
}

fn idle_surface() -> (
    super::types::DocumentSurface,
    std::sync::mpsc::Receiver<DocumentWorkerCommand>,
    std::sync::mpsc::Sender<DocumentWorkerEvent>,
) {
    let (command_tx, command_rx) = std::sync::mpsc::sync_channel(1);
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let surface = super::types::DocumentSurface {
        generation: 1,
        source: representative_pdf_source().descriptor(),
        command_tx: Some(command_tx),
        event_rx,
        frame: Some(test_frame()),
        failure: None,
        painter: Default::default(),
        loading: false,
        command_in_flight: false,
        pending_commands: Default::default(),
        viewport: None,
    };
    (surface, command_rx, event_tx)
}

#[test]
fn document_surface_preserves_commands_until_each_frame_arrives() {
    let ctx = eframe::egui::Context::default();
    let (mut surface, command_rx, _event_tx) = idle_surface();

    surface.set_fit(DocumentFitMode::Width);
    assert!(surface.command_in_flight);
    assert_eq!(
        command_rx.recv().expect("fit command"),
        DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(DocumentFitMode::Width))
    );
    surface.queue(DocumentWorkerCommand::Viewer(
        DocumentViewerCommand::SetZoom(1.25),
    ));
    assert!(!surface.pending_commands.is_empty());

    surface.apply_event(
        &ctx,
        DocumentWorkerEvent::Frame {
            generation: surface.generation,
            frame: Box::new(test_frame()),
            session_event: DocumentSessionEvent::None,
        },
    );
    surface.poll(&ctx);

    assert_eq!(
        command_rx.recv().expect("queued zoom command"),
        DocumentWorkerCommand::Viewer(DocumentViewerCommand::SetZoom(1.25))
    );
    assert!(surface.failure.is_none());
    assert!(surface.pending_commands.is_empty());
}

#[test]
fn document_worker_applies_queued_commands_in_order() {
    let generation = 7;
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    command_tx
        .send(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Fit(
            DocumentFitMode::Width,
        )))
        .expect("fit command");
    command_tx
        .send(DocumentWorkerCommand::Viewer(
            DocumentViewerCommand::SetZoom(1.25),
        ))
        .expect("zoom command");
    drop(command_tx);
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    super::worker::run(
        generation,
        representative_pdf_source(),
        command_rx,
        event_tx,
        eframe::egui::Context::default(),
    );

    let events = event_rx.try_iter().collect::<Vec<_>>();
    assert_eq!(events.len(), 3);
    let session_events = events
        .into_iter()
        .map(|event| match event {
            DocumentWorkerEvent::Frame {
                generation: event_generation,
                frame,
                session_event,
            } => {
                assert_eq!(event_generation, generation);
                assert_eq!(frame.format, ViewerDocumentFormat::Pdf);
                session_event
            }
            DocumentWorkerEvent::Failure { failure, .. } => {
                panic!("document worker failed: {failure}")
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        session_events,
        vec![
            DocumentSessionEvent::None,
            DocumentSessionEvent::Viewer(DocumentViewerEvent::FitChanged(DocumentFitMode::Width)),
            DocumentSessionEvent::Viewer(DocumentViewerEvent::ZoomChanged(1.25)),
        ]
    );
}

#[test]
fn document_surface_preserves_a_command_when_the_worker_channel_is_full() {
    let (mut surface, _initial_command_rx, _event_tx) = idle_surface();

    surface.command_tx.take();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    sender
        .send(DocumentWorkerCommand::Viewer(DocumentViewerCommand::Next))
        .expect("prefill command channel");
    surface.command_tx = Some(sender);
    surface.command_in_flight = false;

    let preserved = DocumentWorkerCommand::Viewer(DocumentViewerCommand::Previous);
    surface.send(preserved);

    assert_eq!(surface.pending_commands.take_next(), Some(preserved));
    assert!(!surface.command_in_flight);
    drop(receiver);
}
