use eframe::egui;

use super::{DocumentSurface, DocumentSurfaceSource};
use crate::preview_pane::PreviewPane;

impl PreviewPane {
    pub(crate) fn full_render_document_source(
        &mut self,
        source: DocumentSurfaceSource,
        force: bool,
    ) {
        if !force
            && self
                .document_surface
                .as_ref()
                .is_some_and(|surface| surface.source() == &source)
        {
            return;
        }
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.render_rx = None;
        self.is_loading = false;
        self.html_browser = None;
        self.document_failure = None;
        self.sections.clear();
        self.section_lifecycle.clear();
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.session_generation = self.session_generation.saturating_add(1);
        let context = self.repaint_ctx.clone().unwrap_or_default();
        self.document_surface = Some(DocumentSurface::start(source, &context));
    }

    pub(crate) fn has_document_surface(&self) -> bool {
        self.document_surface.is_some() || self.document_failure.is_some()
    }

    pub(crate) fn show_document_surface(&mut self, ui: &mut egui::Ui) {
        if let Some(surface) = &mut self.document_surface {
            surface.show(ui);
        } else if let Some(failure) = &self.document_failure {
            super::render_support::show_failure(ui, failure);
        }
    }

    pub(crate) fn full_render_document_failure(&mut self, failure: super::DocumentFailure) {
        failure.log();
        self.cancel_token
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.render_rx = None;
        self.is_loading = false;
        self.html_browser = None;
        self.document_surface = None;
        self.sections.clear();
        self.section_lifecycle.clear();
        self.outline_items.clear();
        self.anchor_map.clear();
        self.document_anchors.clear();
        self.document_failure = Some(failure);
    }

    pub(crate) fn document_frame_state_for_test(&self) -> Option<(String, usize, usize, String)> {
        self.document_surface.as_ref()?.frame_state_for_test()
    }

    pub(crate) fn document_failure_for_test(&self) -> Option<String> {
        self.document_failure
            .as_ref()
            .map(super::DocumentFailure::details)
            .or_else(|| {
                self.document_surface
                    .as_ref()
                    .and_then(DocumentSurface::failure_details_for_test)
            })
    }

    pub(crate) fn document_is_idle_for_test(&self) -> Option<bool> {
        self.document_surface
            .as_ref()
            .map(DocumentSurface::is_idle_for_test)
    }

    pub(crate) fn document_next_for_test(&mut self) -> bool {
        self.document_surface
            .as_mut()
            .is_some_and(DocumentSurface::next_for_test)
    }
}
