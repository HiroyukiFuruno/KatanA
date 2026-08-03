use katana_document_viewer::{
    DocumentSurfaceCommand, DocumentViewerState, DocumentViewport, SpreadsheetGridSurface,
    SpreadsheetViewerSession, ViewerCapabilities, ViewerDiagnostic,
};

use super::source::DocumentSurfaceSource;
use super::types::{DocumentFailure, DocumentFailureLayer, DocumentFrame};
use super::worker::DocumentWorkerCommand;
use super::worker_support::{INITIAL_VIEWPORT_HEIGHT, INITIAL_VIEWPORT_WIDTH, failure};

pub(super) struct SpreadsheetRuntime {
    source: DocumentSurfaceSource,
    session: SpreadsheetViewerSession,
    surface: SpreadsheetGridSurface,
    state: DocumentViewerState,
    capabilities: ViewerCapabilities,
    diagnostics: Vec<ViewerDiagnostic>,
    viewport: DocumentViewport,
}

impl SpreadsheetRuntime {
    pub(super) fn new(
        source: &DocumentSurfaceSource,
        session: SpreadsheetViewerSession,
    ) -> Result<Self, DocumentFailure> {
        let artifact = session.artifact();
        let state = DocumentViewerState::new(artifact.sheet_count);
        let capabilities = artifact.capabilities.clone();
        let diagnostics = artifact.diagnostics.clone();
        let viewport = DocumentViewport::new(INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);
        let surface = spreadsheet_surface(source, &session, 0, viewport)?;
        Ok(Self {
            source: source.clone(),
            session,
            surface,
            state,
            capabilities,
            diagnostics,
            viewport,
        })
    }

    pub(super) fn apply(&mut self, command: DocumentWorkerCommand) -> Result<(), DocumentFailure> {
        match command {
            DocumentWorkerCommand::Viewer(command) => self.apply_viewer_command(command)?,
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Resize(viewport)) => {
                self.resize(viewport)?;
            }
            DocumentWorkerCommand::Surface(DocumentSurfaceCommand::Grid(command)) => {
                let _ = self.surface.apply_command(command);
            }
        }
        Ok(())
    }

    pub(super) fn frame(&mut self) -> Result<DocumentFrame, DocumentFailure> {
        self.materialize()?;
        Ok(DocumentFrame {
            surface: self.surface.frame(),
            state: self.state,
            capabilities: self.capabilities.clone(),
            diagnostics: self.diagnostics.clone(),
            format: self.source.format,
        })
    }

    fn apply_viewer_command(
        &mut self,
        command: katana_document_viewer::DocumentViewerCommand,
    ) -> Result<(), DocumentFailure> {
        let previous = self.state.active_index;
        self.state.apply(command).map_err(|error| {
            failure(
                &self.source,
                "control",
                DocumentFailureLayer::KdvWorker,
                error,
            )
        })?;
        if self.state.active_index != previous {
            self.replace_surface()?;
        }
        Ok(())
    }

    fn resize(&mut self, viewport: DocumentViewport) -> Result<(), DocumentFailure> {
        self.viewport = viewport;
        self.replace_surface()
    }

    fn replace_surface(&mut self) -> Result<(), DocumentFailure> {
        self.surface = spreadsheet_surface(
            &self.source,
            &self.session,
            self.state.active_index,
            self.viewport,
        )?;
        Ok(())
    }

    fn materialize(&mut self) -> Result<(), DocumentFailure> {
        let coordinates = self.surface.materialization_request();
        let cells = self
            .session
            .materialize_cells(self.surface.sheet_index(), coordinates)
            .map_err(|error| {
                failure(
                    &self.source,
                    "materialize",
                    DocumentFailureLayer::KdvWorker,
                    error,
                )
            })?;
        self.surface.supply_cells(cells).map_err(|error| {
            failure(
                &self.source,
                "update grid surface",
                DocumentFailureLayer::KdvSurface,
                error,
            )
        })
    }
}

fn spreadsheet_surface(
    source: &DocumentSurfaceSource,
    session: &SpreadsheetViewerSession,
    sheet_index: usize,
    viewport: DocumentViewport,
) -> Result<SpreadsheetGridSurface, DocumentFailure> {
    let sheet = session.artifact().sheets.get(sheet_index).ok_or_else(|| {
        failure(
            source,
            "open sheet",
            DocumentFailureLayer::KdvWorker,
            "spreadsheet sheet is missing",
        )
    })?;
    SpreadsheetGridSurface::new(sheet, viewport).map_err(|error| {
        failure(
            source,
            "create grid surface",
            DocumentFailureLayer::KdvSurface,
            error,
        )
    })
}
