use std::{path::Path, sync::atomic::AtomicBool};

use eframe::egui;
use katana_document_viewer::DocumentGridSurfaceFrame;

use super::super::{
    painter_grid_style,
    painter_tests::{grid_cell, grid_surface},
};
use crate::font_loader::{
    NormalizeFonts,
    office_faces::{FontFaceRequest, FontFaceResolver, ResolvedFontFace},
    office_font_leases::{DocumentFontLease, DocumentFontLeaseManager},
};

pub(super) const VIEWPORT: egui::Vec2 = egui::vec2(240.0, 80.0);

pub(super) fn resolve_face(
    family: &str,
    path: &Path,
    bold: bool,
    italic: bool,
) -> ResolvedFontFace {
    let candidates = [(
        "font fixture".to_owned(),
        path.to_string_lossy().into_owned(),
    )];
    let requests = [FontFaceRequest {
        family: family.to_owned(),
        bold,
        italic,
    }];
    let resolved = FontFaceResolver::resolve(&candidates, &requests, &AtomicBool::new(false));
    assert!(
        resolved.diagnostics.is_empty(),
        "font fixture resolves cleanly"
    );
    assert_eq!(resolved.faces.len(), 1, "exact face request resolves");
    resolved
        .faces
        .into_iter()
        .next()
        .expect("one resolved face")
}

#[cfg(test)]
pub(super) fn font_lease(
    context: &crate::test_ui::Context,
    faces: &[ResolvedFontFace],
) -> DocumentFontLease {
    let base = NormalizeFonts::new(egui::FontDefinitions::default())
        .normalize(&[])
        .into_inner();
    let manager = DocumentFontLeaseManager::install_base(context, std::sync::Arc::new(base));
    let mut lease = manager.lease(context);
    lease.replace_faces(faces);
    lease
}

pub(super) fn styled_cell(
    family: &str,
    bold: bool,
    italic: bool,
) -> katana_document_viewer::DocumentGridCell {
    let mut cell = grid_cell();
    cell.text = "Glyph proof".to_owned();
    cell.appearance.font_family = family.to_owned();
    cell.appearance.bold = bold;
    cell.appearance.italic = italic;
    cell
}

pub(super) fn paint_and_capture(
    context: &crate::test_ui::Context,
    cell: &katana_document_viewer::DocumentGridCell,
    lease: &DocumentFontLease,
) -> (Vec<std::sync::Arc<egui::Galley>>, usize) {
    let mut frame = grid_surface();
    frame.cells[0] = cell.clone();
    paint_frame_and_capture(context, &frame, Some(lease))
}

pub(super) fn paint_frame_and_capture(
    context: &crate::test_ui::Context,
    frame: &DocumentGridSurfaceFrame,
    lease: Option<&DocumentFontLease>,
) -> (Vec<std::sync::Arc<egui::Galley>>, usize) {
    let output = context.run_ui(frame_input(), |ui| paint_grid(ui, frame, lease));
    (galleys(&output.shapes), mesh_indices(context, &output))
}

pub(super) fn frame_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, VIEWPORT)),
        ..Default::default()
    }
}

pub(super) fn paint_grid(
    ui: &egui::Ui,
    frame: &DocumentGridSurfaceFrame,
    lease: Option<&DocumentFontLease>,
) {
    painter_grid_style::paint_grid(
        ui,
        egui::Rect::from_min_size(egui::Pos2::ZERO, VIEWPORT),
        frame,
        lease,
    );
}

pub(super) fn galleys(shapes: &[egui::epaint::ClippedShape]) -> Vec<std::sync::Arc<egui::Galley>> {
    shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.clone()),
            _ => None,
        })
        .collect()
}

fn mesh_indices(context: &crate::test_ui::Context, output: &egui::FullOutput) -> usize {
    context
        .tessellate(output.shapes.clone(), output.pixels_per_point)
        .iter()
        .filter_map(|primitive| match &primitive.primitive {
            egui::epaint::Primitive::Mesh(mesh) => Some(mesh.indices.len()),
            egui::epaint::Primitive::Callback(_) => None,
        })
        .sum()
}
