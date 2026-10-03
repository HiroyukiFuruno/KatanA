use super::*;
use egui::{FontData, FontId};
use std::sync::Arc;

#[test]
fn build_definitions_registers_metadata_families_of_loaded_fonts() {
    let fonts =
        crate::font_loader::SystemFontLoader::build_font_definitions(&[], &[], &[], None, None)
            .into_inner();
    assert_eq!(
        fonts.families[&FontFamily::Name("Ubuntu".into())][0],
        "Ubuntu-Light"
    );
    assert_eq!(fonts.families[&FontFamily::Name("Hack".into())][0], "Hack");
}

fn renamed_default_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (old, new) in [("Ubuntu-Light", "payload-a"), ("Hack", "payload-b")] {
        let data = fonts.font_data.remove(old).expect("default real font");
        fonts.font_data.insert(new.to_owned(), data);
        rename_family_key(&mut fonts, old, new);
    }
    fonts
}

fn rename_family_key(fonts: &mut FontDefinitions, old: &str, new: &str) {
    for key in fonts.families.values_mut().flatten() {
        if key == old {
            *key = new.to_owned();
        }
    }
}

#[test]
fn registers_metadata_names_and_resolves_them_in_egui() {
    let mut fonts = renamed_default_fonts();
    let proportional = fonts.families[&FontFamily::Proportional].clone();
    let monospace = fonts.families[&FontFamily::Monospace].clone();
    NamedFontFamiliesOps::register(&mut fonts);
    assert_alias_matches(&fonts, "Ubuntu", "payload-a", &proportional);
    assert_alias_matches(&fonts, "Hack", "payload-b", &monospace);

    let context = egui::Context::default();
    context.set_fonts(fonts);
    let mut galley = None;
    let mut output = context.run_ui(Default::default(), |ui| {
        let color = ui.visuals().text_color();
        galley = Some(ui.ctx().fonts_mut(|fonts| {
            fonts.layout_no_wrap(
                "Registered family".into(),
                FontId::new(16.0, FontFamily::Name("Ubuntu".into())),
                color,
            )
        }));
    });
    assert!(!galley.expect("layout result").rows.is_empty());
    output.textures_delta.clear();
}

fn assert_alias_matches(fonts: &FontDefinitions, alias: &str, key: &str, chain: &[String]) {
    let registered = &fonts.families[&FontFamily::Name(alias.into())];
    assert_eq!(registered.first().map(String::as_str), Some(key));
    assert_eq!(registered, chain);
}

#[test]
fn preserves_payloads_generic_chains_and_fallback_membership() {
    let mut fonts = renamed_default_fonts();
    let proportional = fonts.families[&FontFamily::Proportional].clone();
    let monospace = fonts.families[&FontFamily::Monospace].clone();
    let byte_total: usize = fonts.font_data.values().map(|data| data.font.len()).sum();
    let pointers: Vec<_> = fonts.font_data.values().map(Arc::as_ptr).collect();
    NamedFontFamiliesOps::register(&mut fonts);

    assert_eq!(fonts.families[&FontFamily::Proportional], proportional);
    assert_eq!(fonts.families[&FontFamily::Monospace], monospace);
    assert_eq!(
        fonts
            .font_data
            .values()
            .map(|data| data.font.len())
            .sum::<usize>(),
        byte_total
    );
    assert!(fonts.font_data.values().map(Arc::as_ptr).eq(pointers));
    assert_eq!(
        fonts.families[&FontFamily::Name("Ubuntu".into())],
        proportional
    );
    assert_eq!(fonts.families[&FontFamily::Name("Hack".into())], monospace);
    assert!(proportional.iter().any(|key| key == "NotoEmoji-Regular"));
}

#[test]
fn preserves_explicit_names_and_is_idempotent() {
    let mut fonts = renamed_default_fonts();
    let explicit = FontFamily::Name("Ubuntu".into());
    fonts
        .families
        .insert(explicit.clone(), vec!["payload-a".into()]);
    NamedFontFamiliesOps::register(&mut fonts);
    let after_first = fonts.families.clone();
    NamedFontFamiliesOps::register(&mut fonts);
    assert_eq!(fonts.families, after_first);
    assert_eq!(fonts.families[&explicit], ["payload-a"]);
}

#[test]
fn ignores_invalid_and_truncated_payloads_without_rewriting_them() {
    let mut fonts = invalid_payload_fonts();
    let before: Vec<_> = fonts
        .font_data
        .iter()
        .map(|(key, data)| (key.clone(), Arc::as_ptr(data), data.font.len()))
        .collect();
    NamedFontFamiliesOps::register(&mut fonts);
    assert!(
        !fonts
            .families
            .contains_key(&FontFamily::Name("Ubuntu".into()))
    );
    assert_eq!(
        fonts
            .font_data
            .iter()
            .map(|(key, data)| (key.clone(), Arc::as_ptr(data), data.font.len()))
            .collect::<Vec<_>>(),
        before
    );
}

fn invalid_payload_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let valid = fonts
        .font_data
        .remove("Ubuntu-Light")
        .expect("default Ubuntu font");
    let truncated = FontData::from_owned(valid.font[..24].to_vec());
    fonts
        .font_data
        .insert("truncated".into(), Arc::new(truncated));
    fonts.font_data.insert(
        "invalid".into(),
        Arc::new(FontData::from_owned(b"not a font".to_vec())),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .expect("default proportional chain")
        .extend(["truncated".into(), "invalid".into()]);
    let mut bad_index = (*valid).clone();
    bad_index.index = u32::MAX;
    fonts
        .font_data
        .insert("bad-index".into(), Arc::new(bad_index));
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .expect("default proportional chain")
        .push("bad-index".into());
    fonts
}
