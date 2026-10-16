use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use eframe::egui;

use super::font_lookup_types::{FontLookupJob, FontLookupResult};
use crate::font_loader::office_faces::{FontFaceRequest, FontFaceResolver};

pub(super) struct FontLookupWorker;

#[path = "font_lookup_scheduler.rs"]
mod scheduler;

impl FontLookupWorker {
    pub(super) fn enqueue(
        cancelled: Arc<AtomicBool>,
        run: Box<dyn FnOnce() + Send + 'static>,
    ) -> std::io::Result<()> {
        scheduler::FontLookupScheduler::enqueue(cancelled, run)
    }

    pub(super) fn start(
        surface_generation: u64,
        lookup_generation: u64,
        requests: Vec<FontFaceRequest>,
        ctx: &egui::Context,
    ) -> std::io::Result<FontLookupJob> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let (events, results) = mpsc::channel();
        let repaint = ctx.clone();
        Self::enqueue(
            Arc::clone(&cancelled),
            Box::new(move || {
                Self::resolve(
                    surface_generation,
                    lookup_generation,
                    requests,
                    worker_cancelled,
                    events,
                    repaint,
                );
            }),
        )?;
        Ok(FontLookupJob { cancelled, results })
    }

    pub(super) fn spawn(
        name: String,
        work: impl FnOnce() + Send + 'static,
    ) -> std::io::Result<std::thread::JoinHandle<()>> {
        super::DocumentWorkerLifecycle::spawn(name, work)
    }

    fn resolve(
        surface_generation: u64,
        lookup_generation: u64,
        requests: Vec<FontFaceRequest>,
        cancelled: Arc<AtomicBool>,
        events: mpsc::Sender<FontLookupResult>,
        repaint: egui::Context,
    ) {
        let started = std::time::Instant::now();
        if cancelled.load(Ordering::Acquire) {
            return;
        }
        let candidates = Self::candidates(surface_generation, lookup_generation);
        let resolve_started = std::time::Instant::now();
        let resolution = FontFaceResolver::resolve(candidates, &requests, &cancelled);
        let resolve_us = resolve_started.elapsed().as_micros();
        Self::log_faces(&resolution.faces);
        super::debug_log::DebugLog::write(
            "document_font_lookup",
            format_args!(
                "surface_generation={surface_generation} lookup_generation={lookup_generation} requests={} faces={} elapsed_us={} resolve_us={resolve_us}",
                requests.len(),
                resolution.faces.len(),
                started.elapsed().as_micros()
            ),
        );
        if !cancelled.load(Ordering::Acquire) {
            let _ = events.send(FontLookupResult {
                surface_generation,
                lookup_generation,
                resolution,
            });
            repaint.request_repaint();
        }
    }

    fn candidates(surface_generation: u64, lookup_generation: u64) -> &'static [(String, String)] {
        let started = std::time::Instant::now();
        super::debug_log::DebugLog::write(
            "document_font_candidates",
            format_args!(
                "surface_generation={surface_generation} lookup_generation={lookup_generation} phase=started"
            ),
        );
        let candidates = katana_platform::os_fonts::OsFontScanner::cached_fonts();
        super::debug_log::DebugLog::write(
            "document_font_candidates",
            format_args!(
                "surface_generation={surface_generation} lookup_generation={lookup_generation} phase=ready candidates={} candidate_lookup_us={}",
                candidates.len(),
                started.elapsed().as_micros()
            ),
        );
        candidates
    }

    fn log_faces(faces: &[crate::font_loader::office_faces::ResolvedFontFace]) {
        for face in faces {
            super::debug_log::DebugLog::write(
                "document_font_face",
                format_args!(
                    "family={:?} weight={} bold={} italic={} path={} index={} bytes={}",
                    face.family,
                    face.weight,
                    face.bold,
                    face.italic,
                    face.path.display(),
                    face.face_index,
                    face.payload.font.len()
                ),
            );
        }
    }
}

#[cfg(test)]
#[path = "font_lookup_worker_tests.rs"]
mod tests;
