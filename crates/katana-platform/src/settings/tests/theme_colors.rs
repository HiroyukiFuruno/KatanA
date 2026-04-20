/* WHY: Verification of theme color logic, including custom overrides and legacy migration. */

use super::*;
use crate::theme::{Rgb, ThemeColors, ThemeMode, ThemePreset};

#[test]
fn test_effective_theme_colors_uses_preset_by_default() {
    let s = AppSettings::default();
    let colors = s.effective_theme_colors();
    assert_eq!(colors, SettingsDefaultOps::select_initial_preset().colors());
}

#[test]
fn test_effective_theme_colors_uses_custom_when_set() {
    let mut s = AppSettings::default();
    let mut custom = ThemePreset::Nord.colors();
    custom.system.background = Rgb {
        r: 10,
        g: 20,
        b: 30,
    };
    s.theme.custom_color_overrides = Some(custom.clone());
    assert_eq!(s.effective_theme_colors(), custom);
}

#[test]
fn test_theme_preset_save_and_restore() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");

    let mut settings = AppSettings::default();
    settings.theme.preset = ThemePreset::Dracula;
    let repo = JsonFileRepository::new(path.clone());
    repo.save(&settings).unwrap();

    let loaded = repo.load();
    assert_eq!(loaded.theme.preset, ThemePreset::Dracula);
    assert!(loaded.theme.custom_color_overrides.is_none());
}

#[test]
fn test_custom_color_overrides_save_and_restore() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");

    let mut settings = AppSettings::default();
    settings.theme.preset = ThemePreset::Nord;
    let mut custom = ThemePreset::Nord.colors();
    custom.system.background = Rgb {
        r: 10,
        g: 20,
        b: 30,
    };
    settings.theme.custom_color_overrides = Some(custom.clone());
    let repo = JsonFileRepository::new(path.clone());
    repo.save(&settings).unwrap();

    let loaded = repo.load();
    assert_eq!(loaded.theme.preset, ThemePreset::Nord);
    assert_eq!(loaded.theme.custom_color_overrides, Some(custom));
    assert_eq!(
        loaded.effective_theme_colors().system.background,
        Rgb {
            r: 10,
            g: 20,
            b: 30
        }
    );
}

#[test]
fn test_select_preset_for_mode_dark() {
    assert_eq!(
        SettingsDefaultOps::select_preset_for_mode(Some(true)),
        ThemePreset::KatanaDark
    );
}

#[test]
fn test_select_preset_for_mode_light() {
    assert_eq!(
        SettingsDefaultOps::select_preset_for_mode(Some(false)),
        ThemePreset::KatanaLight
    );
}

#[test]
fn test_select_preset_for_mode_unknown() {
    assert_eq!(
        SettingsDefaultOps::select_preset_for_mode(None),
        ThemePreset::KatanaDark
    );
}

#[test]
fn test_legacy_theme_colors_migration() {
    let json = r#"{
        "name": "Legacy Dark",
        "mode": "Dark",
        "background":             { "r": 30, "g": 30, "b": 30 },
        "panel_background":       { "r": 37, "g": 37, "b": 38 },
        "text":                   { "r": 212, "g": 212, "b": 212 },
        "text_secondary":         { "r": 180, "g": 180, "b": 180 },
        "accent":                 { "r": 86, "g": 156, "b": 214 },
        "title_bar_text":         { "r": 180, "g": 180, "b": 180 },
        "file_tree_text":         { "r": 220, "g": 220, "b": 220 },
        "active_file_highlight":  { "r": 40, "g": 80, "b": 160, "a": 100 },
        "warning_text":           { "r": 255, "g": 165, "b": 0 },
        "border":                 { "r": 60, "g": 60, "b": 60 },
        "selection":              { "r": 38, "g": 79, "b": 120 },
        "code_background":        { "r": 25, "g": 25, "b": 40 },
        "preview_background":     { "r": 35, "g": 35, "b": 50 }
    }"#;
    let colors: ThemeColors = serde_json::from_str(json).unwrap();
    assert_eq!(colors.name, "Legacy Dark");
    assert_eq!(colors.mode, ThemeMode::Dark);
    assert_eq!(
        colors.system.background,
        Rgb {
            r: 30,
            g: 30,
            b: 30
        }
    );
    assert_eq!(
        colors.code.background,
        Rgb {
            r: 25,
            g: 25,
            b: 40
        }
    );
    assert_eq!(
        colors.preview.background,
        Rgb {
            r: 35,
            g: 35,
            b: 50
        }
    );
}

#[test]
fn test_legacy_theme_colors_migration_light() {
    let json = r#"{
        "name": "Legacy Light",
        "mode": "Light",
        "background":             { "r": 255, "g": 255, "b": 255 },
        "panel_background":       { "r": 240, "g": 240, "b": 240 },
        "text":                   { "r": 30, "g": 30, "b": 30 },
        "text_secondary":         { "r": 100, "g": 100, "b": 100 },
        "accent":                 { "r": 0, "g": 122, "b": 204 },
        "title_bar_text":         { "r": 50, "g": 50, "b": 50 },
        "file_tree_text":         { "r": 40, "g": 40, "b": 40 },
        "active_file_highlight":  { "r": 200, "g": 220, "b": 255, "a": 120 }
    }"#;
    let colors: ThemeColors = serde_json::from_str(json).unwrap();
    assert_eq!(colors.name, "Legacy Light");
    assert_eq!(colors.mode, ThemeMode::Light);
    assert_eq!(
        colors.system.success_text,
        Rgb {
            r: 20,
            g: 160,
            b: 20
        }
    );
    assert_eq!(
        colors.code.line_number_text,
        Rgb {
            r: 160,
            g: 160,
            b: 160
        }
    );
    assert_eq!(
        colors.system.warning_text,
        Rgb {
            r: 255,
            g: 140,
            b: 0
        }
    );
    assert_eq!(
        colors.code.background,
        Rgb {
            r: 30,
            g: 30,
            b: 30
        }
    );
    assert_eq!(
        colors.preview.background,
        Rgb {
            r: 35,
            g: 35,
            b: 35
        }
    );
}

#[test]
fn test_theme_mode_to_theme_string() {
    assert_eq!(ThemeMode::Dark.to_theme_string(), "dark");
    assert_eq!(ThemeMode::Light.to_theme_string(), "light");
}
