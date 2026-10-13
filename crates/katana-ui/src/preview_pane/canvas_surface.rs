use std::sync::Arc;

use eframe::egui;
use katana_core::canvas::CanvasDocument;

mod cards;
mod colors;
mod controls;
mod edges;
mod geometry;
mod labels;
mod notices;
mod render;
mod state;

pub(crate) use labels::CanvasSurfaceLabels;
pub(crate) use state::CanvasLoadError;

use geometry::CanvasLayout;
use state::CanvasLoadState;

pub(crate) struct CanvasSurface {
    state: CanvasLoadState,
    layout: CanvasLayout,
    view_rect: egui::Rect,
    fit_requested: bool,
    was_drawn: bool,
    revision: Option<super::canvas_request::CanvasRevision>,
}

impl Default for CanvasSurface {
    fn default() -> Self {
        Self {
            state: CanvasLoadState::Loading,
            layout: CanvasLayout::default(),
            view_rect: CanvasLayout::empty_bounds(),
            fit_requested: true,
            was_drawn: false,
            revision: None,
        }
    }
}

impl CanvasSurface {
    pub(crate) fn loading(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn set_document(&mut self, document: Arc<CanvasDocument>) {
        self.revision = None;
        self.was_drawn = false;
        match CanvasLayout::new(&document) {
            Ok(layout) => {
                self.layout = layout;
                self.state = CanvasLoadState::Ready(document);
                self.reset_view();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn set_error(&mut self, error: CanvasLoadError) {
        self.revision = None;
        self.was_drawn = false;
        self.state = CanvasLoadState::Failed(error);
        self.layout = CanvasLayout::default();
        self.reset_view();
    }

    pub(crate) fn reset_view(&mut self) {
        self.fit_requested = true;
    }

    pub(crate) fn set_document_revision(
        &mut self,
        document: Arc<CanvasDocument>,
        revision: Option<super::canvas_request::CanvasRevision>,
    ) {
        if revision.is_some()
            && self.revision == revision
            && matches!(self.state, CanvasLoadState::Ready(_))
        {
            return;
        }
        self.set_document(document);
        if matches!(self.state, CanvasLoadState::Ready(_)) {
            self.revision = revision;
        }
    }

    pub(crate) fn is_drawn(&self) -> bool {
        self.was_drawn && !matches!(self.state, CanvasLoadState::Loading)
    }

    #[cfg(test)]
    fn view_rect(&self) -> egui::Rect {
        self.view_rect
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, labels: &CanvasSurfaceLabels<'_>) {
        match &self.state {
            CanvasLoadState::Loading => {
                ui.spinner();
                ui.label(labels.loading);
            }
            CanvasLoadState::Failed(error) => notices::CanvasNotices::error_ui(ui, error, labels),
            CanvasLoadState::Ready(document) => {
                let document = Arc::clone(document);
                self.ready_ui(ui, &document, labels);
            }
        }
        self.was_drawn = true;
    }

    fn ready_ui(
        &mut self,
        ui: &mut egui::Ui,
        document: &CanvasDocument,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        self.controls_ui(ui, labels);
        notices::CanvasNotices::diagnostics_ui(ui, document, labels);
        if document.nodes.is_empty() {
            ui.label(labels.empty);
        }
        if self.fit_requested || !CanvasLayout::valid_rect(self.view_rect) {
            self.view_rect = self.layout.fit_bounds(ui.available_size());
            self.fit_requested = false;
        }
        egui::Scene::new()
            .zoom_range(geometry::MIN_ZOOM..=geometry::MAX_ZOOM)
            .show(ui, &mut self.view_rect, |ui| {
                render::CanvasRenderer::draw_ui(ui, document, &self.layout, labels);
            });
    }
}

#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod host_refresh_tests;
#[cfg(test)]
mod interaction_tests;
#[cfg(test)]
mod render_tests;
