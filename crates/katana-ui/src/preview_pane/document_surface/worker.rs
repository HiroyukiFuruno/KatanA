use eframe::egui;
use katana_document_viewer::{
    DocumentFrame, DocumentSession, DocumentSessionCommand, DocumentSessionConfig,
    DocumentSessionEvent, DocumentViewport, OfficeWorkerConfig,
};
use std::sync::mpsc::{Receiver, Sender};

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer};
use super::worker_support::{
    INITIAL_VIEWPORT_HEIGHT, INITIAL_VIEWPORT_WIDTH, failure, format_extension,
    office_worker_executable,
};

pub(super) type DocumentWorkerCommand = DocumentSessionCommand;

#[derive(Debug)]
pub(super) enum DocumentWorkerEvent {
    Frame {
        generation: u64,
        frame: Box<DocumentFrame>,
        session_event: DocumentSessionEvent,
    },
    Failure {
        generation: u64,
        failure: DocumentFailure,
    },
}

pub(super) fn run(
    generation: u64,
    source: DocumentSurfaceSource,
    commands: Receiver<DocumentWorkerCommand>,
    events: Sender<DocumentWorkerEvent>,
    repaint: egui::Context,
) {
    let mut session = match open_session(source.clone()) {
        Ok(session) => session,
        Err(failure) => {
            send_failure(generation, failure, &events, &repaint);
            return;
        }
    };
    process_commands(
        generation,
        &mut session,
        commands,
        &events,
        &repaint,
        &source,
    );
    session.close();
}

fn open_session(mut source: DocumentSurfaceSource) -> Result<DocumentSession, DocumentFailure> {
    let viewer_source = source.take_viewer_source()?;
    let viewport = DocumentViewport::new(INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);
    let config = session_config(&source, viewport)?;
    DocumentSession::open(viewer_source, config)
        .map_err(|error| failure(&source, "open", DocumentFailureLayer::KdvWorker, error))
}

fn session_config(
    source: &DocumentSurfaceSource,
    viewport: DocumentViewport,
) -> Result<DocumentSessionConfig, DocumentFailure> {
    let config = DocumentSessionConfig::new(viewport);
    if source.format == katana_core::document_source::BinaryDocumentFormat::Pdf {
        return Ok(config);
    }
    Ok(config.office_worker(OfficeWorkerConfig::new(office_worker_executable(source)?)))
}

fn process_commands(
    generation: u64,
    session: &mut DocumentSession,
    commands: Receiver<DocumentWorkerCommand>,
    events: &Sender<DocumentWorkerEvent>,
    repaint: &egui::Context,
    source: &DocumentSurfaceSource,
) {
    if !send_frame(
        generation,
        session,
        DocumentSessionEvent::None,
        events,
        repaint,
        source,
    ) {
        return;
    }
    while let Ok(command) = commands.recv() {
        tracing::debug!(generation, ?command, "applying document command");
        let event = match session.apply(command) {
            Ok(event) => event,
            Err(error) => {
                let failure = failure(source, "input", DocumentFailureLayer::KdvWorker, error);
                send_failure(generation, failure, events, repaint);
                return;
            }
        };
        if !send_frame(generation, session, event, events, repaint, source) {
            return;
        }
    }
}

fn send_frame(
    generation: u64,
    session: &mut DocumentSession,
    session_event: DocumentSessionEvent,
    events: &Sender<DocumentWorkerEvent>,
    repaint: &egui::Context,
    source: &DocumentSurfaceSource,
) -> bool {
    let frame = match session.frame() {
        Ok(frame) => frame,
        Err(error) => {
            let failure = failure(source, "frame", DocumentFailureLayer::KdvSurface, error);
            send_failure(generation, failure, events, repaint);
            return false;
        }
    };
    log_frame(generation, &frame);
    if events
        .send(DocumentWorkerEvent::Frame {
            generation,
            frame: Box::new(frame),
            session_event,
        })
        .is_err()
    {
        return false;
    }
    repaint.request_repaint_after(std::time::Duration::ZERO);
    true
}

fn log_frame(generation: u64, frame: &DocumentFrame) {
    tracing::debug!(
        generation,
        format = format_extension(frame.format),
        active_index = frame.state.active_index,
        item_count = frame.state.item_count,
        surface = ?frame.surface.kind(),
        "produced document frame"
    );
}

fn send_failure(
    generation: u64,
    failure: DocumentFailure,
    events: &Sender<DocumentWorkerEvent>,
    repaint: &egui::Context,
) {
    let _ = events.send(DocumentWorkerEvent::Failure {
        generation,
        failure,
    });
    repaint.request_repaint_after(std::time::Duration::ZERO);
}
