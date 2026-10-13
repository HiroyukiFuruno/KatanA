use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use katana_ui::app_state::*;

#[test]
fn settings_tab_default_is_theme() {
    /* WHY: Verify that the settings dialog starts on the 'Theme' tab by default. */
    assert_eq!(SettingsTab::default(), SettingsTab::Theme);
}

#[test]
fn app_state_show_settings_defaults_to_false() {
    /* WHY: Verify that the settings window is hidden upon application startup. */
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert!(!state.layout.show_settings);
}

#[test]
fn app_state_active_settings_tab_defaults_to_theme() {
    /* WHY: Verify that AppState initialization correctly sets the default settings tab. */
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert_eq!(state.config.active_settings_tab, SettingsTab::Theme);
}

#[test]
fn show_settings_can_be_toggled() {
    /* WHY: Verify that the show_settings layout flag can be manually toggled. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );

    assert!(!state.layout.show_settings);
    state.layout.show_settings = true;
    assert!(state.layout.show_settings);
    state.layout.show_settings = false;
    assert!(!state.layout.show_settings);
}

#[test]
fn active_settings_tab_can_be_changed() {
    /* WHY: Verify that switching between different settings tabs (Theme, Font, Layout) updates the state correctly. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );

    assert_eq!(state.config.active_settings_tab, SettingsTab::Theme);
    state.config.active_settings_tab = SettingsTab::Font;
    assert_eq!(state.config.active_settings_tab, SettingsTab::Font);
    state.config.active_settings_tab = SettingsTab::Layout;
    assert_eq!(state.config.active_settings_tab, SettingsTab::Layout);
}
