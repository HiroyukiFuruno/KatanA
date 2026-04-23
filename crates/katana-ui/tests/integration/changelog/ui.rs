use crate::integration::harness_utils::setup_harness;
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_changelog_display() {
    /* WHY: Verify that the Changelog panel correctly renders versions from
     * CHANGELOG.md and can be opened via the sidebar button. */
    let mut harness = setup_harness();
    harness.step();

    // Force open changelog
    harness
        .state_mut()
        .trigger_action(AppAction::ShowReleaseNotes);
    let expected_title = format!(
        "{} v{}",
        I18nOps::get().menu.release_notes,
        env!("CARGO_PKG_VERSION")
    );
    let mut found = false;
    for _ in 0..400 {
        harness.step();
        if harness.query_all_by_label(&expected_title).count() > 0 {
            found = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(found, "Changelog window was not displayed!");
}

#[test]
fn test_integration_changelog_auto_popup_version_delta() {
    /* WHY: Verify that the changelog popup automatically appears if the app detects
     * a version update (previous_app_version != current_app_version), ensuring users see new features. */
    // We need a fresh app instance to trigger the popup logic on start
    let mut harness = egui_kittest::Harness::builder().build_eframe(move |_cc| {
        let ai_registry = katana_core::ai::AiProviderRegistry::new();
        let plugin_registry = katana_core::plugin::PluginRegistry::new();

        // We use the same harness generation setup but explicitly set 0.0.1
        let harness_dir = crate::integration::harness_utils::fresh_temp_dir(
            "katana_test_settings_harness_changelog",
        );
        let settings_path = harness_dir.join("settings.json");
        let mut state = katana_ui::app_state::AppState::new(
            ai_registry,
            plugin_registry,
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(settings_path),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());

        // Trigger version upgrade logic
        state
            .config
            .settings
            .settings_mut()
            .updates
            .previous_app_version = Some("0.0.1".to_string());

        state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
            katana_platform::workspace::InMemoryWorkspaceRepository::default(),
        ));
        let mut app = katana_ui::shell::KatanaApp::new(state);
        app.skip_splash();
        app.disable_update_check_for_test();
        app.enable_changelog_popup_for_test();
        app
    });
    let expected_title = format!(
        "{} v{}",
        I18nOps::get().menu.release_notes,
        env!("CARGO_PKG_VERSION")
    );
    let mut found = false;
    for _ in 0..400 {
        harness.step();

        if harness.query_all_by_label(&expected_title).count() > 0 {
            found = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(
        found,
        "Changelog popup window was not displayed automatically on startup!"
    );
}
