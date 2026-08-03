use eframe::egui;
use katana_document_viewer::{DocumentSurfaceCommand, DocumentViewerCommand};
use std::sync::mpsc::{Receiver, Sender};

use super::document_runtime::DocumentRuntime;
use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFrame};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum DocumentWorkerCommand {
    Viewer(DocumentViewerCommand),
    Surface(DocumentSurfaceCommand),
}

#[derive(Debug)]
pub(super) enum DocumentWorkerEvent {
    Frame {
        generation: u64,
        frame: Box<DocumentFrame>,
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
    let mut runtime = match DocumentRuntime::open(source) {
        Ok(runtime) => runtime,
        Err(failure) => {
            send_failure(generation, failure, &events, &repaint);
            return;
        }
    };
    if !send_frame(generation, &mut runtime, &events, &repaint) {
        return;
    }
    while let Ok(command) = commands.recv() {
        match runtime.apply(command) {
            Ok(()) if send_frame(generation, &mut runtime, &events, &repaint) => {}
            Ok(()) => return,
            Err(failure) => {
                send_failure(generation, failure, &events, &repaint);
                return;
            }
        }
    }
}

fn send_frame(
    generation: u64,
    runtime: &mut DocumentRuntime,
    events: &Sender<DocumentWorkerEvent>,
    repaint: &egui::Context,
) -> bool {
    let frame = match runtime.frame() {
        Ok(frame) => frame,
        Err(failure) => {
            send_failure(generation, failure, events, repaint);
            return false;
        }
    };
    if events
        .send(DocumentWorkerEvent::Frame {
            generation,
            frame: Box::new(frame),
        })
        .is_err()
    {
        return false;
    }
    repaint.request_repaint_after(std::time::Duration::ZERO);
    true
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
