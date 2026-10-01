use std::sync::atomic::Ordering;
use std::sync::mpsc::TryRecvError;

use eframe::egui;

use super::font_lookup_types::{FontLookupFailure, FontLookupJob, FontLookupResult};
use super::font_lookup_worker::FontLookupWorker;
use crate::font_loader::office_faces::{FontFaceRequest, FontFaceResolutionDiagnostic};
use crate::font_loader::office_font_leases::{DocumentFontLease, DocumentFontLeaseManager};

#[derive(Default)]
pub(super) struct DocumentFontLookup {
    surface_generation: u64,
    lookup_generation: u64,
    requests: Vec<FontFaceRequest>,
    pending: Option<FontLookupJob>,
    lease: Option<DocumentFontLease>,
    diagnostics: Vec<FontFaceResolutionDiagnostic>,
    failure: Option<FontLookupFailure>,
}

impl DocumentFontLookup {
    pub(super) fn update(
        &mut self,
        surface_generation: u64,
        requests: Vec<FontFaceRequest>,
        ctx: &egui::Context,
    ) {
        if self.surface_generation == surface_generation && self.requests == requests {
            return;
        }
        self.cancel();
        self.surface_generation = surface_generation;
        self.lookup_generation = self
            .lookup_generation
            .checked_add(1)
            .expect("font generation");
        self.requests = requests;
        self.diagnostics.clear();
        self.failure = None;
        if self.requests.is_empty() {
            self.lease.take();
            return;
        }
        if self.acquire_lease(ctx) {
            self.start_pending(ctx);
        }
    }

    fn start_pending(&mut self, ctx: &egui::Context) {
        match FontLookupWorker::start(
            self.surface_generation,
            self.lookup_generation,
            self.requests.clone(),
            ctx,
        ) {
            Ok(job) => self.pending = Some(job),
            Err(error) => self.fail(FontLookupFailure::WorkerSpawn(error.kind())),
        }
    }

    fn acquire_lease(&mut self, ctx: &egui::Context) -> bool {
        if self.lease.is_some() {
            return true;
        }
        let Some(manager) = DocumentFontLeaseManager::from_context(ctx) else {
            self.fail(FontLookupFailure::RegistryUnavailable);
            return false;
        };
        self.lease = Some(manager.lease(ctx));
        true
    }

    pub(super) fn lease(&self) -> Option<&DocumentFontLease> {
        self.lease.as_ref()
    }

    pub(super) fn poll(&mut self, surface_generation: u64) {
        let Some(job) = &self.pending else {
            return;
        };
        match job.results.try_recv() {
            Ok(result) => {
                self.accept(surface_generation, result);
                self.pending.take();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.pending.take();
                self.fail(FontLookupFailure::WorkerDisconnected);
            }
        }
    }

    fn accept(&mut self, surface_generation: u64, result: FontLookupResult) -> bool {
        if result.surface_generation != surface_generation
            || result.surface_generation != self.surface_generation
            || result.lookup_generation != self.lookup_generation
        {
            return false;
        }
        let Some(lease) = &mut self.lease else {
            return false;
        };
        lease.replace_faces(&result.resolution.faces);
        self.diagnostics = result.resolution.diagnostics;
        self.log_diagnostics();
        true
    }

    fn log_diagnostics(&self) {
        for diagnostic in &self.diagnostics {
            super::debug_log::DebugLog::write(
                "document_font_resolution",
                format_args!(
                    "surface_generation={} lookup_generation={} cause={}",
                    self.surface_generation,
                    self.lookup_generation,
                    diagnostic.description()
                ),
            );
        }
    }

    fn fail(&mut self, failure: FontLookupFailure) {
        self.lease.take();
        super::debug_log::DebugLog::write(
            "document_font_lookup_failure",
            format_args!(
                "surface_generation={} lookup_generation={} cause={}",
                self.surface_generation,
                self.lookup_generation,
                failure.description()
            ),
        );
        self.failure = Some(failure);
    }

    pub(super) fn cancel(&mut self) {
        if let Some(job) = self.pending.take() {
            job.cancelled.store(true, Ordering::Release);
        }
    }
}

impl Drop for DocumentFontLookup {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
#[path = "font_lookup_tests.rs"]
mod tests;
