/* WHY: Verification of the settings service logic, including OS-level integrations. */

use super::*;
use crate::theme::ThemePreset;

#[test]
fn test_settings_service_new_loads_from_repository() {
    let svc = SettingsService::new(Box::new(InMemoryRepository));
    assert_eq!(
        svc.settings().theme.theme,
        SettingsDefaultOps::default_theme()
    );
}

#[test]
fn test_settings_service_save_delegates_to_repository() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("svc.json");
    let mut svc = SettingsService::new(Box::new(JsonFileRepository::new(path.clone())));
    svc.settings_mut().theme.theme = "light".to_string();
    svc.save().unwrap();

    let loaded = JsonFileRepository::new(path).load();
    assert_eq!(loaded.theme.theme, "light");
}

#[test]
fn test_settings_service_default_uses_in_memory() {
    let svc = SettingsService::default();
    assert_eq!(
        svc.settings().theme.theme,
        SettingsDefaultOps::default_theme()
    );
    assert!(svc.save().is_ok());
}

#[test]
fn test_apply_os_default_theme_is_noop_for_existing_users() {
    let mut service = SettingsService::new(Box::new(InMemoryRepository));
    service.settings_mut().theme.preset = ThemePreset::Dracula;
    service.apply_os_default_theme();
    assert_eq!(
        service.settings().theme.preset,
        ThemePreset::Dracula,
        "existing user's preset must not be overwritten"
    );
}

#[test]
fn test_apply_os_default_theme_on_first_launch_picks_katana_preset() {
    let repo = FirstLaunchRepo {
        preset: ThemePreset::KatanaDark,
    };
    let mut service = SettingsService::new(Box::new(repo));
    service.apply_os_default_theme();
    let preset = &service.settings().theme.preset;
    assert!(
        *preset == ThemePreset::KatanaDark || *preset == ThemePreset::KatanaLight,
        "first launch must yield KatanaDark or KatanaLight, got {preset:?}"
    );
}

#[test]
fn test_apply_os_default_language_is_noop_for_existing_users() {
    let mut service = SettingsService::new(Box::new(InMemoryRepository));
    service.settings_mut().language = "ja".to_string();
    service.apply_os_default_language();
    assert_eq!(service.settings().language, "ja");

    service.apply_os_default_language();
    assert_eq!(service.settings().language, "ja");
}

#[test]
fn test_apply_os_default_language_sets_auto_on_first_launch() {
    let repo = FirstLaunchRepo {
        preset: ThemePreset::KatanaDark,
    };
    let mut service = SettingsService::new(Box::new(repo));

    service.apply_os_default_language();
    assert_eq!(service.settings().language, AUTO_LANGUAGE_CODE);
}

#[test]
fn test_resolve_effective_language_uses_os_locale_for_auto() {
    let mut service = SettingsService::default();
    service.settings_mut().language = AUTO_LANGUAGE_CODE.to_string();

    assert_eq!(
        service.resolve_effective_language(|| Some("ja-JP".to_string())),
        "ja"
    );
    assert_eq!(
        service.resolve_effective_language(|| Some("en-US".to_string())),
        "en"
    );
    assert_eq!(
        service.resolve_effective_language(|| Some("unknown".to_string())),
        "en"
    );
    assert_eq!(service.resolve_effective_language(|| None), "en");
}

#[test]
fn test_resolve_effective_language_preserves_existing_japanese_language() {
    let mut service = SettingsService::default();
    service.settings_mut().language = "ja".to_string();

    assert_eq!(
        service.resolve_effective_language(|| Some("en-US".to_string())),
        "ja"
    );
}
