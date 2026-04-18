use katana_platform::settings::SettingsDefaultOps;
use katana_platform::theme::{Rgb, ThemeMode, ThemePreset};
use katana_platform::{InMemoryRepository, SettingsService};
use katana_ui::theme_bridge::ThemeBridgeOps;

#[test]
fn switching_preset_changes_effective_colors() {
    /* WHY: Verify that changing the ThemePreset in the settings service accurately updates the
     * effective ThemeColors used for UI derivation. */
    let mut svc = SettingsService::new(Box::new(InMemoryRepository::default()));

    let default_colors = svc.settings().effective_theme_colors();
    let expected_default_mode = SettingsDefaultOps::select_initial_preset().colors().mode;
    assert_eq!(default_colors.mode, expected_default_mode);

    svc.settings_mut().theme.preset = ThemePreset::Dracula;
    let dracula_colors = svc.settings().effective_theme_colors();
    assert_eq!(dracula_colors.name, "Dracula");
    assert_eq!(dracula_colors.mode, ThemeMode::Dark);
    assert_ne!(
        dracula_colors.system.background,
        default_colors.system.background
    );

    svc.settings_mut().theme.preset = ThemePreset::KatanaLight;
    let light_colors = svc.settings().effective_theme_colors();
    assert_eq!(light_colors.mode, ThemeMode::Light);
}

#[test]
fn switching_preset_changes_visuals() {
    /* WHY: Verify that ThemeBridgeOps correctly generates egui::Visuals from the effective
     * colors after a preset change (e.g., verifying dark_mode flag transition). */
    let mut svc = SettingsService::new(Box::new(InMemoryRepository::default()));

    let default_visuals =
        ThemeBridgeOps::visuals_from_theme(&svc.settings().effective_theme_colors());
    let expected_dark =
        SettingsDefaultOps::select_initial_preset().colors().mode == ThemeMode::Dark;
    assert_eq!(default_visuals.dark_mode, expected_dark);

    svc.settings_mut().theme.preset = ThemePreset::GitHubLight;
    let light_visuals =
        ThemeBridgeOps::visuals_from_theme(&svc.settings().effective_theme_colors());
    assert!(!light_visuals.dark_mode);

    let expected_panel = ThemePreset::GitHubLight.colors().system.panel_background;
    assert_eq!(
        light_visuals.panel_fill,
        eframe::egui::Color32::from_rgb(expected_panel.r, expected_panel.g, expected_panel.b)
    );
}

#[test]
fn custom_overrides_take_precedence_over_preset() {
    /* WHY: Verify that the custom_color_overrides field in settings correctly masks preset values,
     * allowing for fine-grained user branding without losing the underlying theme structure. */
    let mut svc = SettingsService::new(Box::new(InMemoryRepository::default()));

    svc.settings_mut().theme.preset = ThemePreset::Nord;
    let mut custom = ThemePreset::Nord.colors();
    custom.system.background = Rgb { r: 1, g: 2, b: 3 };
    svc.settings_mut().theme.custom_color_overrides = Some(custom);

    let effective = svc.settings().effective_theme_colors();
    assert_eq!(effective.system.background, Rgb { r: 1, g: 2, b: 3 });
    assert_eq!(
        effective.system.accent,
        ThemePreset::Nord.colors().system.accent
    );
}
