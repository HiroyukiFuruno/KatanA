use eframe::egui;

use super::font_test_support::{
    font_lease, paint_and_capture, paint_frame_and_capture, resolve_face, styled_cell,
};

#[test]
fn real_bold_face_drives_layout_and_paint_without_faux_bold() {
    let (family, _, path) = crate::font_loader::office_faces::installed_regular_bold_pair();
    let face = resolve_face(&family, &path, true, false);
    assert!(face.bold, "fixture must be a real bold face");
    assert!(!face.italic, "fixture must not be an italic face");
    let context = crate::test_ui::Context::default();
    let lease = font_lease(&context, &[face]);
    let expected = lease
        .family_for(&family, true, false)
        .expect("leased bold face");
    let cell = styled_cell(&family, true, false);
    let (galleys, meshes) = paint_and_capture(&context, &cell, &lease);

    assert_eq!(galleys.len(), 1, "real bold must not be drawn twice");
    assert_eq!(galleys[0].job.sections[0].format.font_id.family, expected);
    assert!(!galleys[0].job.sections[0].format.italics);
    assert!(meshes > 0, "real glyphs tessellate to a mesh");
}

#[test]
fn regular_face_does_not_suppress_faux_italic_or_bold() {
    let (family, path, _) = crate::font_loader::office_faces::installed_regular_bold_pair();
    let face = resolve_face(&family, &path, false, false);
    assert!(!face.bold && !face.italic, "fixture must be a regular face");
    let context = crate::test_ui::Context::default();
    let lease = font_lease(&context, &[face]);
    assert!(lease.family_for(&family, true, false).is_none());
    assert!(lease.family_for(&family, false, true).is_none());
    let cell = styled_cell(&family, true, true);
    let (galleys, meshes) = paint_and_capture(&context, &cell, &lease);

    assert_eq!(galleys.len(), 2, "regular-only face retains faux bold");
    assert!(galleys[0].job.sections[0].format.italics);
    assert!(meshes > 0, "fallback glyphs tessellate to a mesh");
}

#[test]
fn same_frame_face_registration_falls_back_until_next_frame() {
    let (family, _, path) = crate::font_loader::office_faces::installed_regular_bold_pair();
    let face = resolve_face(&family, &path, true, false);
    let context = crate::test_ui::Context::default();
    let mut lease = font_lease(&context, &[]);
    let mut frame = super::super::painter_tests::grid_surface();
    frame.cells[0] = styled_cell(&family, true, false);
    assert_same_frame_fallback(&context, &mut lease, &face, &frame);
    assert_next_frame_bold(&context, &lease, &family, &frame);
}

fn assert_same_frame_fallback(
    context: &crate::test_ui::Context,
    lease: &mut crate::font_loader::office_font_leases::DocumentFontLease,
    face: &crate::font_loader::office_faces::ResolvedFontFace,
    frame: &katana_document_viewer::DocumentGridSurfaceFrame,
) {
    let output = context.run_ui(super::font_test_support::frame_input(), |ui| {
        lease.replace_faces(std::slice::from_ref(face));
        super::font_test_support::paint_grid(ui, frame, Some(lease));
    });
    let galleys = super::font_test_support::galleys(&output.shapes);
    assert_eq!(galleys.len(), 2);
    assert_eq!(
        galleys[0].job.sections[0].format.font_id.family,
        egui::FontFamily::Proportional
    );
}

fn assert_next_frame_bold(
    context: &crate::test_ui::Context,
    lease: &crate::font_loader::office_font_leases::DocumentFontLease,
    family: &str,
    frame: &katana_document_viewer::DocumentGridSurfaceFrame,
) {
    let (galleys, _) = paint_frame_and_capture(context, frame, Some(lease));
    assert_eq!(galleys.len(), 1);
    assert_eq!(
        galleys[0].job.sections[0].format.font_id.family,
        lease
            .family_for(family, true, false)
            .expect("leased bold face")
    );
}
