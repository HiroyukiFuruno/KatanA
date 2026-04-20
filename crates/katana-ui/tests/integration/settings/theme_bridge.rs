use katana_platform::theme::{Rgb, Rgba, ThemeMode, ThemePreset};
use katana_ui::theme_bridge::ThemeBridgeOps;

#[test]
fn dark_preset_produces_dark_visuals() {
    /* WHY: Verify that the KatanaDark preset correctly initializes egui::Visuals with dark_mode enabled. */
    let colors = ThemePreset::KatanaDark.colors();
    let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
    assert!(visuals.dark_mode);
}

#[test]
fn light_preset_produces_light_visuals() {
    /* WHY: Verify that the KatanaLight preset correctly initializes egui::Visuals with dark_mode disabled. */
    let colors = ThemePreset::KatanaLight.colors();
    let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
    assert!(!visuals.dark_mode);
}

#[test]
fn panel_fill_matches_theme_panel_bg() {
    /* WHY: Verify that the system's panel background color is correctly mapped to egui's panel_fill property. */
    let colors = ThemePreset::Nord.colors();
    let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
    assert_eq!(
        visuals.panel_fill,
        ThemeBridgeOps::rgb_to_color32(colors.system.panel_background)
    );
}

#[test]
fn text_color_override_is_not_set() {
    /* WHY: Stability check: ensure that the theme engine relies on widget-level fg_stroke instead of global text color overrides,
     * which can break semantic highlighting in labels. */
    let colors = ThemePreset::Dracula.colors();
    let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
    assert_eq!(visuals.override_text_color, None);
    assert_eq!(
        visuals.widgets.noninteractive.fg_stroke.color,
        ThemeBridgeOps::rgb_to_color32(colors.system.text)
    );
}

#[test]
fn all_presets_produce_valid_visuals() {
    /* WHY: Integrity check: Iterate through every built-in theme preset and verify they all produce valid, non-overriding egui::Visuals. */
    for preset in ThemePreset::builtins() {
        let colors = preset.colors();
        let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
        assert_eq!(
            visuals.override_text_color,
            None,
            "{}: override_text_color must be None",
            preset.display_name()
        );
        let is_dark = colors.mode == ThemeMode::Dark;
        assert_eq!(visuals.dark_mode, is_dark, "{}", preset.display_name());
    }
}

#[test]
fn rgb_to_color32_converts_correctly() {
    /* WHY: Verify the baseline color conversion utility for Rgb types. */
    let c = Rgb {
        r: 255,
        g: 128,
        b: 0,
    };
    assert_eq!(
        ThemeBridgeOps::rgb_to_color32(c),
        eframe::egui::Color32::from_rgb(255, 128, 0)
    );
}

#[test]
fn rgba_to_color32_converts_correctly() {
    /* WHY: Verify the baseline color conversion utility for Rgba types (alpha-aware). */
    let c = Rgba {
        r: 40,
        g: 80,
        b: 160,
        a: 100,
    };
    assert_eq!(
        ThemeBridgeOps::rgba_to_color32(c),
        eframe::egui::Color32::from_rgba_unmultiplied(40, 80, 160, 100)
    );
}
