use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::state::command_palette::*;
use katana_ui::state::command_palette_providers::*;

#[test]
fn test_app_command_provider() {
    let provider = AppCommandProvider;

    // Empty query returns generic recent/common actions
    let results = provider.search("", None);
    assert!(!results.is_empty());
    assert_eq!(results[0].kind, CommandPaletteResultKind::RecentOrCommon);

    // Match query
    let results = provider.search("Settings", None);
    assert!(!results.is_empty());
    assert_eq!(results[0].kind, CommandPaletteResultKind::Action);
    assert_eq!(results[0].label, "Toggle Settings");

    match &results[0].execute_payload {
        CommandPaletteExecutePayload::DispatchAppAction(ref action) => {
            assert!(matches!(action, AppAction::ToggleSettings));
        }
        _ => panic!("Expected DispatchAppAction payload"),
    }
}

#[test]
fn test_integration_command_palette_ui() {
    use egui_kittest::Harness;
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;
    use katana_ui::app_state::{AppAction, AppState};
    use katana_ui::shell::KatanaApp;
    let settings_path =
        std::env::temp_dir().join(format!("katana_test_cp_ui_{}.json", std::process::id()));
    let _ = std::fs::remove_file(&settings_path);

    let sp_clone = settings_path.clone();
    let mut harness = Harness::builder().build_eframe(move |_cc| {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(sp_clone),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());

        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    });

    harness.step();

    // 1. Open Palette
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCommandPalette);
    harness.step();
    harness.step();

    assert!(harness.state().app_state_for_test().command_palette.is_open);

    // 2. Type query (triggers search)
    // We can't easily "type" into TextEdit in kittest yet without focus/keyboard events,
    // but we can mutate the state directly for this integration test.
    harness
        .state_mut()
        .app_state_mut()
        .command_palette
        .current_query = "Settings".into();
    harness.step();

    // 3. Verify results are populated in UI
    // AppCommandProvider should provide "Toggle Settings"
    let _ = harness.get_by_label("Toggle Settings");

    // 4. Select and execution (Enter)
    // We'll simulate the Enter key by checking if the action is dispatched
    // Actually, let's just trigger the keyboard event if possible, or verify selection logic.
    // 4. Select and execution
    // Simulation of clicking the result item in the palette
    // Note: get_by_label matches the Toggle Settings text
    harness.get_by_label("Toggle Settings").click();

    harness.step(); // UI processes click, sets is_open = false, BUT action is not set in interact.clicked() branch in show()!
                    // Wait, looking at command_palette.rs:
                    // if interact.clicked() { is_open = false; }
                    // It DOES NOT set the action in the clicked() branch! It only sets it in the Enter key branch.
                    // This is a bug in the implementation or I'm misreading.

    harness.step();
    harness.step();

    // Palette should be closed
    assert!(!harness.state().app_state_for_test().command_palette.is_open);

    // Settings should be open (ToggleSettings action processed via click)
    assert!(harness.state().app_state_for_test().layout.show_settings);

    let _ = std::fs::remove_file(&settings_path);
}

#[test]
fn test_integration_command_palette_keyboard_navigation() {
    use egui_kittest::Harness;
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;
    use katana_ui::app_state::{AppAction, AppState};
    use katana_ui::shell::KatanaApp;
    let settings_path =
        std::env::temp_dir().join(format!("katana_test_cp_kbd_{}.json", std::process::id()));

    let sp_clone = settings_path.clone();
    let mut harness = Harness::builder().build_eframe(move |_cc| {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(sp_clone),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());

        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    });

    harness.step();

    // 1. Open Palette
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCommandPalette);
    harness.step();

    // 2. Initial selection is 0
    assert_eq!(
        harness
            .state()
            .app_state_for_test()
            .command_palette
            .selected_index,
        0
    );

    // 3. Move Down
    harness.key_press(egui::Key::ArrowDown);
    harness.step();
    assert_eq!(
        harness
            .state()
            .app_state_for_test()
            .command_palette
            .selected_index,
        1
    );

    // 4. Move Up
    harness.key_press(egui::Key::ArrowUp);
    harness.step();
    assert_eq!(
        harness
            .state()
            .app_state_for_test()
            .command_palette
            .selected_index,
        0
    );

    // 5. Confirm (Enter)
    // By default, index 0 is Toggle Settings (from AppCommandProvider when empty)
    harness.key_press(egui::Key::Enter);
    harness.step();
    harness.step();

    assert!(!harness.state().app_state_for_test().command_palette.is_open);
    assert!(harness.state().app_state_for_test().layout.show_settings);

    // 6. Test Dismissal (Escape)
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCommandPalette);
    harness.step();
    assert!(harness.state().app_state_for_test().command_palette.is_open);
    harness.key_press(egui::Key::Escape);
    harness.step();
    assert!(!harness.state().app_state_for_test().command_palette.is_open);

    let _ = std::fs::remove_file(&settings_path);
}

#[test]
fn test_integration_command_palette_provider_availability() {
    use egui_kittest::Harness;
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;
    use katana_ui::app_state::{AppAction, AppState};
    use katana_ui::shell::KatanaApp;
    let settings_path =
        std::env::temp_dir().join(format!("katana_test_cp_avail_{}.json", std::process::id()));

    let sp_clone = settings_path.clone();
    let mut harness = Harness::builder().build_eframe(move |_cc| {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(sp_clone),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());

        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    });

    harness.step();

    // No workspace open -> WorkspaceFileProvider and MarkdownContentProvider should return nothing
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleCommandPalette);
    harness.step();

    // Population occurs in show()
    harness
        .state_mut()
        .app_state_mut()
        .command_palette
        .current_query = "any".into();
    harness.step();

    // Should only have results from AppCommandProvider (if query matches something)
    // Or at least it should not crash.
    let results = &harness.state().app_state_for_test().command_palette.results;
    for res in results {
        // Since no workspace, only Actions or RecentOrCommon should exist
        assert!(
            matches!(
                res.kind,
                CommandPaletteResultKind::Action | CommandPaletteResultKind::RecentOrCommon
            ),
            "Expected only app actions without workspace, got {:?}",
            res.kind
        );
    }

    let _ = std::fs::remove_file(&settings_path);
}

#[test]
fn test_integration_search_modal_remains_functional() {
    use egui_kittest::Harness;
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;
    use katana_ui::app_state::{AppAction, AppState};
    use katana_ui::shell::KatanaApp;
    let settings_path = std::env::temp_dir().join(format!(
        "katana_test_cp_fallback_{}.json",
        std::process::id()
    ));

    let sp_clone = settings_path.clone();
    let mut harness = Harness::builder().build_eframe(move |_cc| {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(sp_clone),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());

        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    });

    harness.step();

    // Toggle Search Modal (legacy fallback)
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSearchModal);
    harness.step();

    assert!(
        harness
            .state()
            .app_state_for_test()
            .layout
            .show_search_modal
    );

    // Verify legacy modal title is present
    let _ = harness.get_by_label(&katana_ui::i18n::get().search.modal_title);

    let _ = std::fs::remove_file(&settings_path);
}

#[test]
fn test_keyboard_navigation_state() {
    let mut state = CommandPaletteState::default();

    state.results = vec![
        CommandPaletteResult {
            id: "1".into(),
            label: "Item 1".into(),
            secondary_label: None,
            score: 1.0,
            kind: CommandPaletteResultKind::Action,
            execute_payload: CommandPaletteExecutePayload::DispatchAppAction(
                AppAction::ToggleSettings,
            ),
        },
        CommandPaletteResult {
            id: "2".into(),
            label: "Item 2".into(),
            secondary_label: None,
            score: 0.9,
            kind: CommandPaletteResultKind::Action,
            execute_payload: CommandPaletteExecutePayload::DispatchAppAction(
                AppAction::ToggleWorkspace,
            ),
        },
    ];

    assert_eq!(state.selected_index, 0);

    state.move_down();
    assert_eq!(state.selected_index, 1);

    // Wrap around or stop? Implemented as wrap around in command_palette.rs
    state.move_down();
    assert_eq!(
        state.selected_index, 0,
        "Should wrap around to zero if implemented that way"
    );

    state.move_up();
    assert_eq!(
        state.selected_index, 1,
        "Should wrap around to last if implemented that way"
    );

    state.move_up();
    assert_eq!(state.selected_index, 0);
}
