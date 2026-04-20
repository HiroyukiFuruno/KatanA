/* WHY: Verification of font discovery, mapping, and customization logic. */

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_build_font_definitions_no_candidates() {
    let fonts = SystemFontLoader::build_font_definitions(&[], &[], &[], None, None).into_inner();
    assert!(!fonts.font_data.contains_key("cjk_font"));
    assert!(!fonts.font_data.contains_key("MyCustomFont"));
}

#[test]
fn test_build_font_definitions_includes_emoji_candidates_for_ui_families() {
    let tmp = TempDir::new().unwrap();
    let font_path = tmp.path().join("emoji.ttf");
    fs::write(&font_path, "").unwrap();
    let path_str = font_path.to_str().unwrap();

    let fonts =
        SystemFontLoader::build_font_definitions(&[], &[], &[path_str], None, None).into_inner();

    let emoji_name = "emoji";
    assert!(
        fonts.font_data.contains_key(emoji_name),
        "preview emoji should be included in global egui font families as fallbacks"
    );
    let prop_list = fonts.families.get(&FontFamily::Proportional).unwrap();
    assert!(prop_list.contains(&emoji_name.to_string()));
    let mono_list = fonts.families.get(&FontFamily::Monospace).unwrap();
    assert!(mono_list.contains(&emoji_name.to_string()));
}

#[test]
fn test_custom_font_injection() {
    let tmp = TempDir::new().unwrap();
    let custom_font_path = tmp.path().join("custom.ttf");
    fs::write(&custom_font_path, "").unwrap();
    let path_str = custom_font_path.to_str().unwrap();

    let fonts = SystemFontLoader::build_font_definitions(
        &[],
        &[],
        &[],
        Some(path_str),
        Some("MyCustomFont"),
    );

    assert!(fonts.fonts().font_data.contains_key("MyCustomFont"));
    let prop_list = fonts
        .fonts()
        .families
        .get(&FontFamily::Proportional)
        .unwrap();
    assert_eq!(prop_list.first().unwrap(), "MyCustomFont");
}

#[test]
fn test_custom_font_injection_invalid_path() {
    let fonts = SystemFontLoader::build_font_definitions(
        &[],
        &[],
        &[],
        Some("/path/does/not/exist.ttf"),
        Some("MyCustomFont"),
    );

    assert!(!fonts.fonts().font_data.contains_key("MyCustomFont"));
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_ui_font_setup_does_register_apple_color_emoji_globally() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let proportional = fonts
        .fonts()
        .families
        .get(&FontFamily::Proportional)
        .expect("proportional family");
    assert!(
        proportional.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
        "UI symbol glyphs should include emoji fonts"
    );

    let monospace = fonts
        .fonts()
        .families
        .get(&FontFamily::Monospace)
        .expect("monospace family");
    assert!(
        monospace.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
        "UI symbol glyphs should include emoji fonts"
    );
}

#[test]
fn test_proportional_primary_y_offset_matches_constant() {
    let tmp = TempDir::new().unwrap();
    let font_path = tmp.path().join("prop.ttf");
    fs::write(&font_path, "").unwrap();
    let path_str = font_path.to_str().unwrap();

    let fonts = SystemFontLoader::build_font_definitions(
        &[path_str], // Force our candidate to be found
        &[],
        &[],
        None,
        None,
    );

    let prop_primary = fonts
        .fonts()
        .families
        .get(&FontFamily::Proportional)
        .and_then(|list| list.first())
        .cloned();

    if let Some(name) = prop_primary {
        let font_data = fonts.fonts().font_data.get(&name).expect("font data");
        let y_offset = font_data.tweak.y_offset_factor;
        let expected = crate::font_loader::PROPORTIONAL_Y_OFFSET_FACTOR;
        assert!(
            (y_offset - expected).abs() < 0.001,
            "Proportional primary font y_offset_factor must match PROPORTIONAL_Y_OFFSET_FACTOR ({}). Got: {}.",
            expected,
            y_offset
        );
    }
}
