use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use super::super::font_lookup_types::{FontLookupJob, FontLookupResult};
use super::DocumentFontLookup;
use crate::font_loader::office_faces::{FontFaceRequest, FontFaceResolution};
use crate::font_loader::office_font_leases::DocumentFontLeaseManager;

const SURFACE_GENERATION: u64 = 41;
const LOOKUP_GENERATION: u64 = 3;
const RESULT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

#[cfg(test)]
fn installed_context() -> egui::Context {
    let ctx = egui::Context::default();
    let base = crate::font_loader::NormalizeFonts::new(egui::FontDefinitions::default())
        .normalize(&[])
        .into_inner();
    DocumentFontLeaseManager::install_base(&ctx, Arc::new(base));
    ctx
}

fn request() -> FontFaceRequest {
    FontFaceRequest {
        family: "uninstalled-office-fixture".into(),
        bold: true,
        italic: false,
    }
}

fn result(surface_generation: u64, lookup_generation: u64) -> FontLookupResult {
    FontLookupResult {
        surface_generation,
        lookup_generation,
        resolution: FontFaceResolution::default(),
    }
}

#[test]
fn stale_surface_and_lookup_generations_do_not_install_fonts() {
    let ctx = installed_context();
    let manager = DocumentFontLeaseManager::from_context(&ctx).expect("registry");
    let mut lookup = DocumentFontLookup {
        surface_generation: SURFACE_GENERATION,
        lookup_generation: LOOKUP_GENERATION,
        lease: Some(manager.lease(&ctx)),
        requests: Vec::new(),
        pending: None,
        diagnostics: Vec::new(),
        failure: None,
    };
    assert!(!lookup.accept(
        SURFACE_GENERATION,
        result(SURFACE_GENERATION + 1, LOOKUP_GENERATION)
    ));
    assert!(!lookup.accept(
        SURFACE_GENERATION,
        result(SURFACE_GENERATION, LOOKUP_GENERATION + 1)
    ));
    assert!(!lookup.accept(
        SURFACE_GENERATION + 1,
        result(SURFACE_GENERATION, LOOKUP_GENERATION)
    ));
    assert!(lookup.accept(
        SURFACE_GENERATION,
        result(SURFACE_GENERATION, LOOKUP_GENERATION)
    ));
}

#[test]
fn same_requests_do_not_restart_background_lookup() {
    let ctx = egui::Context::default();
    let mut lookup = DocumentFontLookup::default();
    lookup.update(SURFACE_GENERATION, vec![request()], &ctx);
    assert!(matches!(
        lookup.failure,
        Some(super::super::font_lookup_types::FontLookupFailure::RegistryUnavailable)
    ));
    let generation = lookup.lookup_generation;
    lookup.update(SURFACE_GENERATION, vec![request()], &ctx);
    assert_eq!(lookup.lookup_generation, generation);
    assert!(lookup.pending.is_none());
}

#[test]
fn close_and_empty_request_cancel_pending_jobs_without_joining() {
    let ctx = installed_context();
    let cancelled = Arc::new(AtomicBool::new(false));
    let (_events, results) = mpsc::channel();
    let mut lookup = DocumentFontLookup {
        surface_generation: 0,
        lookup_generation: 0,
        requests: Vec::new(),
        lease: None,
        diagnostics: Vec::new(),
        failure: None,
        pending: Some(FontLookupJob {
            cancelled: cancelled.clone(),
            results,
        }),
    };
    lookup.update(SURFACE_GENERATION, Vec::new(), &ctx);
    assert!(cancelled.load(Ordering::Acquire));
    assert!(lookup.pending.is_none());
    assert!(lookup.lease().is_none());
    let close_cancelled = Arc::new(AtomicBool::new(false));
    let (_events, results) = mpsc::channel();
    lookup.pending = Some(FontLookupJob {
        cancelled: close_cancelled.clone(),
        results,
    });
    drop(lookup);
    assert!(close_cancelled.load(Ordering::Acquire));
}

#[test]
fn disconnected_job_is_typed_and_does_not_block_the_ui() {
    let ctx = installed_context();
    let manager = DocumentFontLeaseManager::from_context(&ctx).expect("registry");
    let (events, results) = mpsc::channel();
    drop(events);
    let mut lookup = DocumentFontLookup {
        surface_generation: 0,
        lookup_generation: 0,
        requests: Vec::new(),
        diagnostics: Vec::new(),
        failure: None,
        lease: Some(manager.lease(&ctx)),
        pending: Some(FontLookupJob {
            cancelled: Arc::new(AtomicBool::new(false)),
            results,
        }),
    };
    lookup.poll(SURFACE_GENERATION);
    assert!(matches!(
        lookup.failure,
        Some(super::super::font_lookup_types::FontLookupFailure::WorkerDisconnected)
    ));
    assert!(lookup.pending.is_none());
    assert!(lookup.lease().is_none());
    let mut output = ctx.run_ui(Default::default(), |ui| {
        ui.label(
            &crate::i18n::I18nOps::get()
                .preview
                .spreadsheet_filter
                .loading,
        );
    });
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();
}

#[test]
fn real_background_worker_returns_result_with_identity_while_ui_paints() {
    let ctx = installed_context();
    let (family, _, _) = crate::font_loader::office_faces::installed_regular_bold_pair();
    let request = FontFaceRequest {
        family,
        bold: true,
        italic: false,
    };
    let job = super::super::font_lookup_worker::FontLookupWorker::start(
        SURFACE_GENERATION,
        LOOKUP_GENERATION,
        vec![request.clone()],
        &ctx,
    )
    .expect("actual font lookup thread");
    let mut output = ctx.run_ui(Default::default(), |ui| {
        ui.label(
            &crate::i18n::I18nOps::get()
                .preview
                .spreadsheet_filter
                .loading,
        );
    });
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();
    let result = job
        .results
        .recv_timeout(RESULT_DEADLINE)
        .expect("worker result");
    assert_eq!(result.surface_generation, SURFACE_GENERATION);
    assert_eq!(result.lookup_generation, LOOKUP_GENERATION);
    assert_eq!(
        result.resolution.faces.len(),
        1,
        "{:#?}",
        result.resolution.diagnostics
    );
    assert_eq!(result.resolution.faces[0].request, request);
    assert!(result.resolution.faces[0].bold);
}
