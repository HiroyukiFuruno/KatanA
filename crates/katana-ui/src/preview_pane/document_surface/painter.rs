use eframe::egui;
use katana_document_viewer::{DocumentSurfaceCommand, DocumentSurfaceFrame, DocumentSurfaceKind};

#[derive(Default)]
pub(super) struct DocumentFramePainter {
    pub(super) texture: Option<egui::TextureHandle>,
    pub(super) texture_fingerprint: Option<String>,
}

impl std::fmt::Debug for DocumentFramePainter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DocumentFramePainter")
            .field("texture_fingerprint", &self.texture_fingerprint)
            .finish_non_exhaustive()
    }
}

pub(super) fn paint_document_frame(
    painter: &mut DocumentFramePainter,
    ui: &mut egui::Ui,
    frame: &DocumentSurfaceFrame,
    surface_id: u64,
) -> Vec<DocumentSurfaceCommand> {
    match frame.kind() {
        DocumentSurfaceKind::Page => super::painter_page::paint(painter, ui, frame, surface_id),
        DocumentSurfaceKind::Grid => super::painter_grid::paint(ui, frame),
    }
}
