use super::super::super::office_faces::{FontFaceRequest, ResolvedFontFace};
use super::super::DocumentFontLeaseManager;
use egui::{FontData, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

pub(super) fn installed_context() -> (
    egui::Context,
    DocumentFontLeaseManager,
    Arc<FontDefinitions>,
) {
    let context = egui::Context::default();
    let base = Arc::new(FontDefinitions::default());
    let manager = DocumentFontLeaseManager::install_base(&context, base.clone());
    (context, manager, base)
}

pub(super) fn face(payload: &Arc<FontData>, bold: bool) -> ResolvedFontFace {
    face_with(payload, "Office Sans", bold, false, false)
}

pub(super) fn face_with(
    payload: &Arc<FontData>,
    family: &str,
    bold: bool,
    italic: bool,
    monospaced: bool,
) -> ResolvedFontFace {
    ResolvedFontFace {
        request: FontFaceRequest {
            family: family.into(),
            bold,
            italic,
        },
        family: "Ubuntu".into(),
        weight: if bold { 700 } else { 400 },
        bold,
        italic,
        monospaced,
        path: "fixture/ubuntu.ttf".into(),
        face_index: 0,
        payload: payload.clone(),
        payload_digest: ResolvedFontFace::digest_payload(payload),
    }
}

pub(super) fn assert_payload(
    context: &egui::Context,
    family: &FontFamily,
    payload: &Arc<FontData>,
) {
    let definitions = context_definitions(context);
    let FontFamily::Name(key) = family else {
        panic!("lease alias must be named");
    };
    assert!(Arc::ptr_eq(&definitions.font_data[key.as_ref()], payload));
}

pub(super) fn assert_aliases(
    context: &egui::Context,
    first: &super::super::DocumentFontLease,
    second: &super::super::DocumentFontLease,
    payload: &Arc<FontData>,
) -> (FontFamily, FontFamily) {
    let regular = first
        .family_for("office sans", false, false)
        .expect("regular alias");
    let shared = second
        .family_for("Office Sans", false, false)
        .expect("shared alias");
    let bold = second
        .family_for("Office Sans", true, false)
        .expect("bold-style alias");
    assert_eq!(regular, shared);
    assert_ne!(regular, bold);
    assert_payload(context, &regular, payload);
    assert_payload(context, &bold, payload);
    (shared, bold)
}

pub(super) fn assert_active_aliases(
    context: &egui::Context,
    base: &FontDefinitions,
    families: [&FontFamily; 2],
    payload: &Arc<FontData>,
) {
    let definitions = context_definitions(context);
    for family in families {
        assert_payload(context, family, payload);
        let chain = &definitions.families[family];
        assert_eq!(&chain[1..], base.families[&FontFamily::Proportional]);
    }
    assert_eq!(
        definitions.families[&FontFamily::Proportional],
        base.families[&FontFamily::Proportional]
    );
}

pub(super) fn assert_same_definitions(context: &egui::Context, expected: &FontDefinitions) {
    let actual = context_definitions(context);
    assert_eq!(actual.families, expected.families);
    assert_eq!(actual.font_data.len(), expected.font_data.len());
    for (key, payload) in &expected.font_data {
        assert!(Arc::ptr_eq(payload, &actual.font_data[key]));
    }
}

pub(super) fn context_definitions(context: &egui::Context) -> FontDefinitions {
    let mut definitions = None;
    let mut output = context.run_ui(Default::default(), |ui| {
        definitions = Some(ui.ctx().fonts_mut(|fonts| fonts.definitions().clone()));
    });
    output.textures_delta.clear();
    definitions.expect("Context fonts initialized in frame")
}

pub(super) fn epoch(manager: &DocumentFontLeaseManager) -> u64 {
    manager.state.lock().expect("lease state").epoch
}

pub(super) fn alias_key(lease: &super::super::DocumentFontLease, bold: bool) -> String {
    let FontFamily::Name(name) = lease
        .family_for("Office Sans", bold, false)
        .expect("active family")
    else {
        panic!("lease family must be named");
    };
    name.to_string()
}

pub(super) fn assert_layout(context: &egui::Context, family: FontFamily) {
    let mut galley = None;
    let mut output = context.run_ui(Default::default(), |ui| {
        let color = ui.visuals().text_color();
        galley = Some(ui.ctx().fonts_mut(|fonts| {
            fonts.layout_no_wrap(
                "Office text".into(),
                FontId::new(16.0, family.clone()),
                color,
            )
        }));
    });
    assert!(!galley.expect("layout result").rows.is_empty());
    output.textures_delta.clear();
}

pub(super) fn base_with_marker(base: &FontDefinitions) -> FontDefinitions {
    let mut updated = base.clone();
    updated
        .families
        .get_mut(&FontFamily::Proportional)
        .expect("proportional chain")
        .push("Hack".into());
    updated
}
