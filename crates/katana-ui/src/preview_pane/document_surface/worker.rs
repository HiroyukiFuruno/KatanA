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
        font_requests: Vec<crate::font_loader::office_faces::FontFaceRequest>,
        session_event: DocumentSessionEvent,
        spreadsheet_metadata: Option<SpreadsheetFrameMetadata>,
        filter_event: Option<SpreadsheetFilterEvent>,
    },
    Failure {
        generation: u64,
        failure: DocumentFailure,
    },
}

#[path = "worker_output.rs"]
mod worker_output;
use worker_output::WorkerOutput;

pub(super) fn run(
    generation: u64,
    mut source: DocumentSurfaceSource,
    commands: Receiver<DocumentWorkerCommand>,
    events: Sender<DocumentWorkerEvent>,
    repaint: egui::Context,
    cancellation: super::worker_cancellation::DocumentWorkerCancellation,
) {
    let output = WorkerOutput {
        events,
        repaint,
        cancellation,
    };
    if output.cancellation.is_cancelled() {
        return;
    }
    let started_at = std::time::Instant::now();
    let mut session = match super::worker_session::open_session(&mut source, &output.cancellation) {
        Ok(Some(session)) => session,
        Ok(None) => return,
        Err(failure) => {
            super::worker_border_projection::send_failure(
                generation,
                failure,
                &output.events,
                &output.repaint,
            );
            return;
        }
    };
    super::debug_log::DebugLog::write(
        "document_session_opened",
        format_args!(
            "generation={} format={} elapsed_ms={} cancelled={} uri={}",
            generation,
            source.format.extension(),
            started_at.elapsed().as_millis(),
            output.cancellation.is_cancelled(),
            source.uri
        ),
    );
    /* WHY: open中にタブが閉じられていたら、不要な初回rasterと罫線準備へ進まない。 */
    if !output.cancellation.is_cancelled() {
        process_commands(generation, &mut session, commands, &output, &source);
    }
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
    output: &WorkerOutput,
    source: &DocumentSurfaceSource,
) {
    if !output.send_frame(
        generation,
        session,
        DocumentSessionEvent::None,
        None,
        source,
    ) {
        return;
    }
    while let Ok(command) = commands.recv() {
        if output.cancellation.is_cancelled() {
            return;
        }
        tracing::debug!(generation, ?command, "applying document command");
        let (event, filter_event) = match worker_commands::apply(session, command) {
            Ok(result) => result,
            Err(error) => {
                let failure = failure(source, "input", DocumentFailureLayer::KdvWorker, error);
                super::worker_border_projection::send_failure(
                    generation,
                    failure,
                    &output.events,
                    &output.repaint,
                );
                return;
            }
        };
        if !output.send_frame(generation, session, event, filter_event, source) {
            return;
        }
    }
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
            super::super::worker_session::open_session(&mut source, &Default::default())
                .expect("PDF session")
                .expect("active PDF session");
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
