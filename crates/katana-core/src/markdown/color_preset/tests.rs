use super::*;

#[test]
fn dark_preset_has_transparent_background() {
    assert_eq!(DiagramColorPreset::dark().background, "transparent");
}

#[test]
fn dark_preset_text_is_light() {
    assert_eq!(DiagramColorPreset::dark().text, "#E0E0E0");
}

#[test]
fn parse_hex_rgb_valid() {
    assert_eq!(
        DiagramColorPreset::parse_hex_rgb("#E0E0E0"),
        Some((224, 224, 224))
    );
}

#[test]
fn luminance_white_is_one() {
    let lum = DiagramColorPreset::relative_luminance("#FFFFFF").unwrap();
    assert!((lum - 1.0).abs() < 0.01);
}
