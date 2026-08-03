use katana_document_viewer::{
    DocumentFitMode, DocumentSurfaceCommand, DocumentSurfaceFrame, DocumentViewerCommand,
    DocumentViewerState, OfficeStaticViewerSession, PdfPageRenderRequest, PdfViewerSession,
    ViewerCapabilities, ViewerDiagnostic,
};

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentFrame};
use super::worker::DocumentWorkerCommand;
use super::worker_support::{
    INITIAL_VIEWPORT_HEIGHT, INITIAL_VIEWPORT_WIDTH, MAX_DOCUMENT_RENDER_SCALE,
    MIN_DOCUMENT_RENDER_SCALE, failure,
};

enum PagedEngine {
    Pdf(PdfViewerSession),
    Office(OfficeStaticViewerSession),
}

const VIEWPORT_HORIZONTAL_CHROME: u32 = 32;
const VIEWPORT_VERTICAL_CHROME: u32 = 72;

pub(super) struct PagedRuntime {
    source: DocumentSurfaceSource,
    engine: PagedEngine,
    state: DocumentViewerState,
    capabilities: ViewerCapabilities,
    diagnostics: Vec<ViewerDiagnostic>,
    item_sizes: Vec<(f32, f32)>,
    viewport: (u32, u32),
}

impl PagedRuntime {
    pub(super) fn from_pdf(source: &DocumentSurfaceSource, session: PdfViewerSession) -> Self {
        let artifact = session.artifact();
        let item_count = artifact.page_count;
        let capabilities = artifact.capabilities.clone();
        let diagnostics = artifact.diagnostics.clone();
        let item_sizes = artifact
            .pages
            .iter()
            .map(|page| (page.width, page.height))
            .collect();
        Self::new(
            source,
            PagedEngine::Pdf(session),
            item_count,
            capabilities,
            diagnostics,
            item_sizes,
        )
    }

    pub(super) fn from_office(
        source: &DocumentSurfaceSource,
        session: OfficeStaticViewerSession,
    ) -> Self {
        let artifact = session.artifact();
        let item_count = artifact.item_count;
        let capabilities = artifact.capabilities.clone();
        let diagnostics = artifact.diagnostics.clone();
        let item_sizes = artifact
            .items
            .iter()
            .map(|item| (item.width, item.height))
            .collect();
        Self::new(
            source,
            PagedEngine::Office(session),
            item_count,
            capabilities,
            diagnostics,
            item_sizes,
        )
    }

    fn new(
        source: &DocumentSurfaceSource,
        engine: PagedEngine,
        item_count: usize,
        capabilities: ViewerCapabilities,
        diagnostics: Vec<ViewerDiagnostic>,
        item_sizes: Vec<(f32, f32)>,
    ) -> Self {
        let mut state = DocumentViewerState::new(item_count);
        let _ = state.apply(DocumentViewerCommand::Fit(DocumentFitMode::Page));
        Self {
            source: source.clone(),
            engine,
            state,
            capabilities,
            diagnostics,
            item_sizes,
            viewport: (INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT),
        }
    }

    pub(super) fn apply(&mut self, command: DocumentWorkerCommand) -> Result<(), DocumentFailure> {
        match command {
            DocumentWorkerCommand::Viewer(command) => {
                self.state.apply(command).map(|_| ()).map_err(|error| {
                    failure(
                        &self.source,
                        "control",
                        DocumentFailureLayer::KdvWorker,
                        error,
                    )
                })
            }
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Resize(viewport)) => {
                self.viewport = (viewport.width, viewport.height);
                Ok(())
            }
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(_)) => Ok(()),
        }
    }

    pub(super) fn frame(&mut self) -> Result<DocumentFrame, DocumentFailure> {
        let request = PdfPageRenderRequest::new(self.state.active_index, self.render_scale());
        let rendered = match &mut self.engine {
            PagedEngine::Pdf(session) => session.render_page(request),
            PagedEngine::Office(session) => session.render_item(request),
        }
        .map_err(|error| {
            failure(
                &self.source,
                "render",
                DocumentFailureLayer::KdvWorker,
                error,
            )
        })?;
        let surface = DocumentSurfaceFrame::from_rendered_page("Document page", rendered).map_err(
            |error| {
                failure(
                    &self.source,
                    "create page surface",
                    DocumentFailureLayer::KdvSurface,
                    error,
                )
            },
        )?;
        Ok(DocumentFrame {
            surface,
            state: self.state,
            capabilities: self.capabilities.clone(),
            diagnostics: self.diagnostics.clone(),
            format: self.source.format,
        })
    }

    fn render_scale(&self) -> f32 {
        let Some((width, height)) = self.item_sizes.get(self.state.active_index).copied() else {
            return self
                .state
                .zoom
                .clamp(MIN_DOCUMENT_RENDER_SCALE, MAX_DOCUMENT_RENDER_SCALE);
        };
        let available_width = self.viewport.0.saturating_sub(VIEWPORT_HORIZONTAL_CHROME) as f32;
        let available_height = self.viewport.1.saturating_sub(VIEWPORT_VERTICAL_CHROME) as f32;
        let fit_width = available_width / width.max(1.0);
        let fit_page = fit_width.min(available_height / height.max(1.0));
        match self.state.fit {
            Some(DocumentFitMode::Width) => fit_width,
            Some(DocumentFitMode::Page) => fit_page,
            None => self.state.zoom,
        }
        .clamp(MIN_DOCUMENT_RENDER_SCALE, MAX_DOCUMENT_RENDER_SCALE)
    }
}
