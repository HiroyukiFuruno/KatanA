use eframe::egui;
use katana_document_viewer::{
    DocumentFrame, DocumentSession, DocumentSessionEvent, SpreadsheetFilterEvent,
    SpreadsheetFrameMetadata,
};
use std::sync::mpsc::{Receiver, Sender};

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer};
use super::worker_support::failure;

#[path = "worker_commands.rs"]
mod worker_commands;
pub(super) use worker_commands::DocumentWorkerCommand;

#[derive(Debug)]
pub(super) enum DocumentWorkerEvent {
    Frame {
        generation: u64,
        frame: Box<DocumentFrame>,
        border_cache: super::painter_grid_borders::PreparedGridBorders,
        session_event: DocumentSessionEvent,
        spreadsheet_metadata: Option<SpreadsheetFrameMetadata>,
        filter_event: Option<SpreadsheetFilterEvent>,
    },
    Failure {
        generation: u64,
        failure: DocumentFailure,
    },
}

pub(super) fn run(
    generation: u64,
    mut source: DocumentSurfaceSource,
    commands: Receiver<DocumentWorkerCommand>,
    events: Sender<DocumentWorkerEvent>,
    repaint: egui::Context,
) {
    let started_at = std::time::Instant::now();
    let mut session = match super::worker_session::open_session(&mut source) {
        Ok(session) => session,
        Err(failure) => {
            super::worker_border_projection::send_failure(generation, failure, &events, &repaint);
            return;
        }
    };
    super::debug_log::DebugLog::write(
        "document_session_opened",
        format_args!(
            "generation={} format={} elapsed_ms={} uri={}",
            generation,
            source.format.extension(),
            started_at.elapsed().as_millis(),
            source.uri
        ),
    );
    process_commands(
        generation,
        &mut session,
        commands,
        &events,
        &repaint,
        &source,
    );
    session.close();
    super::debug_log::DebugLog::write(
        "document_session_closed",
        format_args!(
            "generation={} elapsed_ms={} uri={}",
            generation,
            started_at.elapsed().as_millis(),
            source.uri
        ),
    );
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
        None,
        events,
        repaint,
        source,
    ) {
        return;
    }
    while let Ok(command) = commands.recv() {
        tracing::debug!(generation, ?command, "applying document command");
        let (event, filter_event) = match worker_commands::apply(session, command) {
            Ok(result) => result,
            Err(error) => {
                let failure = failure(source, "input", DocumentFailureLayer::KdvWorker, error);
                super::worker_border_projection::send_failure(generation, failure, events, repaint);
                return;
            }
        };
        if !send_frame(
            generation,
            session,
            event,
            filter_event,
            events,
            repaint,
            source,
        ) {
            return;
        }
    }
}

fn send_frame(
    generation: u64,
    session: &mut DocumentSession,
    session_event: DocumentSessionEvent,
    filter_event: Option<SpreadsheetFilterEvent>,
    events: &Sender<DocumentWorkerEvent>,
    repaint: &egui::Context,
    source: &DocumentSurfaceSource,
) -> bool {
    let frame = match session.frame() {
        Ok(frame) => frame,
        Err(error) => {
            let failure = failure(source, "frame", DocumentFailureLayer::KdvSurface, error);
            super::worker_border_projection::send_failure(generation, failure, events, repaint);
            return false;
        }
    };
    let border_cache = match super::worker_border_projection::project(&frame, generation) {
        Ok(cache) => cache,
        Err(error) => {
            let failure = failure(
                source,
                "frame border projection",
                DocumentFailureLayer::KatanaHost,
                error,
            );
            super::worker_border_projection::send_failure(generation, failure, events, repaint);
            return false;
        }
    };
    let spreadsheet_metadata = session.spreadsheet_frame_metadata();
    super::worker_border_projection::log_frame(generation, &frame);
    if events
        .send(DocumentWorkerEvent::Frame {
            generation,
            frame: Box::new(frame),
            border_cache,
            session_event,
            spreadsheet_metadata,
            filter_event,
        })
        .is_err()
    {
        return false;
    }
    repaint.request_repaint_after(std::time::Duration::ZERO);
    true
}

#[cfg(test)]
mod tests {
    use super::DocumentSurfaceSource;

    #[test]
    fn opened_session_consumes_worker_source_bytes_but_keeps_identity() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.pdf");
        let mut source = DocumentSurfaceSource::local(&fixture).expect("PDF source");
        let identity = source.descriptor();
        assert!(source.byte_len() > 0);
        let mut session =
            super::super::worker_session::open_session(&mut source).expect("PDF session");
        assert_eq!(source.byte_len(), 0);
        assert_eq!(source.uri, identity.uri);
        assert_eq!(source.revision, identity.revision);
        let frame = session.frame().expect("PDF frame after ownership transfer");
        assert_eq!(
            frame.format,
            katana_document_viewer::ViewerDocumentFormat::Pdf
        );
        session.close();
    }
}
