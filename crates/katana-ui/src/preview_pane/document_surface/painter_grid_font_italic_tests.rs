use super::font_test_support::{font_lease, paint_and_capture, resolve_face, styled_cell};

#[test]
fn real_italic_face_drives_layout_and_paint_without_faux_italic() {
    let (family, path) = crate::font_loader::office_faces::installed_italic_face();
    let face = resolve_face(&family, &path, false, true);
    assert!(
        !face.bold && face.italic,
        "fixture must be a real italic face"
    );
    let context = crate::test_ui::Context::default();
    let lease = font_lease(&context, &[face]);
    let expected = lease
        .family_for(&family, false, true)
        .expect("leased italic face");
    let (galleys, meshes) = paint_and_capture(&context, &styled_cell(&family, false, true), &lease);

    assert_eq!(galleys.len(), 1, "real italic must not be synthesized");
    assert_eq!(galleys[0].job.sections[0].format.font_id.family, expected);
    assert!(!galleys[0].job.sections[0].format.italics);
    assert!(meshes > 0, "real italic glyphs tessellate to a mesh");
}

#[test]
fn real_bold_with_italic_request_keeps_only_faux_italic() {
    let (family, _, path) = crate::font_loader::office_faces::installed_regular_bold_pair();
    let face = resolve_face(&family, &path, true, false);
    let context = crate::test_ui::Context::default();
    let lease = font_lease(&context, &[face]);
    let expected = lease
        .family_for(&family, true, false)
        .expect("leased bold face");
    let cell = styled_cell(&family, true, true);
    let (galleys, meshes) = paint_and_capture(&context, &cell, &lease);

    assert_eq!(galleys.len(), 1, "real bold must suppress faux bold draw");
    assert_eq!(galleys[0].job.sections[0].format.font_id.family, expected);
    assert!(galleys[0].job.sections[0].format.italics);
    assert!(meshes > 0, "bold fallback glyphs tessellate to a mesh");
}

#[test]
fn real_italic_with_bold_request_keeps_only_faux_bold() {
    let (family, path) = crate::font_loader::office_faces::installed_italic_face();
    let face = resolve_face(&family, &path, false, true);
    let context = crate::test_ui::Context::default();
    let lease = font_lease(&context, &[face]);
    let expected = lease
        .family_for(&family, false, true)
        .expect("leased italic face");
    let cell = styled_cell(&family, true, true);
    let (galleys, meshes) = paint_and_capture(&context, &cell, &lease);

    assert_eq!(galleys.len(), 2, "missing real bold must retain faux bold");
    assert_eq!(galleys[0].job.sections[0].format.font_id.family, expected);
    assert!(!galleys[0].job.sections[0].format.italics);
    assert!(meshes > 0, "italic fallback glyphs tessellate to a mesh");
}
