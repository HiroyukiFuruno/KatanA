use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

use eframe::egui;

use super::{DocumentFailure, DocumentSurfaceSource};
use crate::preview_pane::PreviewPane;

#[cfg(test)]
#[path = "local_intake_tests.rs"]
mod tests;

const INTAKE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

#[derive(Debug)]
pub(crate) struct LocalDocumentIntake {
    path: PathBuf,
    result: Receiver<Result<DocumentSurfaceSource, DocumentFailure>>,
    force: bool,
    refresh_requested: bool,
}

impl LocalDocumentIntake {
    fn start(path: PathBuf, force: bool, context: egui::Context) -> Result<Self, DocumentFailure> {
        let worker_path = path.clone();
        let (sender, result) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("katana-document-intake".to_owned())
            .spawn(move || {
                let source = DocumentSurfaceSource::local(&worker_path);
                if sender.send(source).is_ok() {
                    context.request_repaint();
                }
            })
            .map_err(|error| {
                DocumentFailure::new(
                    super::types::DocumentFailureLayer::KatanaHost,
                    "start intake",
                    path.display().to_string(),
                    katana_core::document_source::BinaryDocumentFormat::from_path(&path),
                    error.to_string(),
                )
            })?;
        Ok(Self {
            path,
            result,
            force,
            refresh_requested: false,
        })
    }
}

impl PreviewPane {
    pub(crate) fn full_render_document_path(&mut self, path: &Path, force: bool) {
        if let Some(pending) = &mut self.document_intake
            && pending.path == path
        {
            pending.force |= force;
            pending.refresh_requested = true;
            return;
        }
        let context = self.repaint_ctx.clone().unwrap_or_default();
        let pending = match LocalDocumentIntake::start(path.to_path_buf(), force, context) {
            Ok(pending) => pending,
            Err(error) => {
                self.full_render_document_failure(error);
                return;
            }
        };
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.render_rx = None;
        self.html_browser = None;
        self.document_failure = None;
        self.is_loading = true;
        self.document_intake = Some(pending);
    }

    pub(super) fn poll_document_intake(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.document_intake.take() else {
            return;
        };
        match pending.result.try_recv() {
            Ok(result) => {
                self.is_loading = false;
                match result {
                    Ok(source) => self.full_render_document_source(source, pending.force),
                    Err(error) => self.full_render_document_failure(error),
                }
                if pending.refresh_requested {
                    self.full_render_document_path(&pending.path, false);
                }
            }
            Err(TryRecvError::Empty) => {
                self.document_intake = Some(pending);
                ctx.request_repaint_after(INTAKE_POLL_INTERVAL);
            }
            Err(TryRecvError::Disconnected) => {
                self.full_render_document_failure(DocumentFailure::intake(
                    "read",
                    &pending.path,
                    katana_core::document_source::BinaryDocumentFormat::from_path(&pending.path),
                    "document intake stopped before returning a result",
                ));
            }
        }
    }
}
