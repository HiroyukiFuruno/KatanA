use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};

use eframe::egui;

use super::DocumentFailure;
use super::source_revision::SourceRevision;
use crate::preview_pane::PreviewPane;

#[path = "local_intake_pool.rs"]
mod pool;
#[path = "local_intake_request.rs"]
mod request;
#[path = "local_intake_result.rs"]
mod result;
#[path = "local_intake_worker.rs"]
mod worker;

#[cfg(test)]
#[path = "local_intake_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "local_intake_revision_tests.rs"]
mod revision_tests;

#[cfg(all(test, unix))]
#[path = "local_intake_resource_tests.rs"]
mod resource_tests;

#[cfg(all(test, unix))]
#[path = "local_intake_capacity_tests.rs"]
mod capacity_tests;

const INTAKE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);

#[derive(Debug)]
pub(crate) struct LocalDocumentIntake {
    path: PathBuf,
    request: Arc<request::IntakeRequest>,
    result: Receiver<request::IntakeResult>,
    force: bool,
    refresh_requested: bool,
}

impl LocalDocumentIntake {
    fn start(path: PathBuf, force: bool) -> Result<Self, DocumentFailure> {
        Self::start_known(path, force, None)
    }

    fn start_known(
        path: PathBuf,
        force: bool,
        known: Option<SourceRevision>,
    ) -> Result<Self, DocumentFailure> {
        let (request, result) = if let Some(known) = known.filter(|_| !force) {
            request::IntakeRequest::with_revision(path.clone(), Some(known))
        } else {
            request::IntakeRequest::new(path.clone())
        };
        pool::IntakePool::global()
            .enqueue(&request)
            .map_err(|cause| request.failure(cause))?;
        Ok(Self {
            path,
            request,
            result,
            force,
            refresh_requested: false,
        })
    }
}

impl Drop for LocalDocumentIntake {
    fn drop(&mut self) {
        self.request.cancel();
    }
}

impl PreviewPane {
    pub(crate) fn full_render_document_path(&mut self, path: &Path, force: bool) {
        self.clear_canvas_surface();
        if let Some(pending) = &mut self.document_intake
            && pending.path == path
        {
            pending.force |= force;
            pending.refresh_requested = true;
            return;
        }
        self.document_intake = None;
        let known = self
            .document_surface
            .as_ref()
            .filter(|_| !force)
            .map(SourceRevision::from_surface);
        let started = if known.is_some() {
            LocalDocumentIntake::start_known(path.to_path_buf(), force, known)
        } else {
            LocalDocumentIntake::start(path.to_path_buf(), force)
        };
        let pending = match started {
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
        if let Some(cause) = pool::IntakePool::global().failure_cause() {
            self.full_render_document_failure(pending.request.failure(cause));
            return;
        }
        self.poll_document_intake_result(pending, ctx);
    }

    fn poll_document_intake_result(&mut self, pending: LocalDocumentIntake, ctx: &egui::Context) {
        match pending.result.try_recv() {
            Ok(result) => self.finish_document_intake(pending, result),
            Err(TryRecvError::Empty) => {
                self.continue_document_intake(pending, ctx);
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

    fn continue_document_intake(&mut self, pending: LocalDocumentIntake, ctx: &egui::Context) {
        let pool = pool::IntakePool::global();
        if let Some(cause) = pool.failure_cause() {
            self.full_render_document_failure(pending.request.failure(cause));
        } else if let Some(cause) = pool.capacity_failure_cause(&pending.request) {
            self.finish_or_fail_capacity(pending, cause, ctx);
        } else {
            self.document_intake = Some(pending);
            ctx.request_repaint_after(INTAKE_POLL_INTERVAL);
        }
    }

    fn finish_or_fail_capacity(
        &mut self,
        pending: LocalDocumentIntake,
        cause: String,
        ctx: &egui::Context,
    ) {
        match pending.result.try_recv() {
            Ok(result) => self.finish_document_intake(pending, result),
            Err(TryRecvError::Empty) => {
                self.full_render_document_failure(pending.request.failure(cause));
            }
            Err(TryRecvError::Disconnected) => self.poll_document_intake_result(pending, ctx),
        }
    }
}
