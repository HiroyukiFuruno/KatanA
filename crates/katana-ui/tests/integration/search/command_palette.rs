use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
// use katana_ui::i18n::I18nOps;
use katana_ui::state::command_palette::*;
use katana_ui::state::command_palette_providers::*;

#[test]
fn test_app_command_provider() {
    /* WHY: Verify that the AppCommandProvider correctly ranks and returns system actions like "Settings". */
    let provider = AppCommandProvider;

    // Empty query returns generic recent/common actions
    let results = provider.search("", None, None);
    assert!(!results.is_empty());
    assert_eq!(results[0].kind, CommandPaletteResultKind::RecentOrCommon);

    let results = provider.search("Settings", None, None);
    assert!(!results.is_empty());
    assert_eq!(results[0].kind, CommandPaletteResultKind::Action);
    assert_eq!(results[0].label, "Settings");

    match &results[0].execute_payload {
        CommandPaletteExecutePayload::DispatchAppAction(action) => {
            assert!(matches!(action, AppAction::ToggleSettings));
        }
        _ => panic!("Expected DispatchAppAction payload"),
    }
}

#[test]
fn test_integration_command_palette_ui() {
    /* WHY: Full UI integration: verify that opening the palette, searching, and clicking a result dispatches the correct action. */
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
    harness
        .state_mut()
        .app_state_mut()
        .command_palette
        .current_query = "> Settings".into();
    harness.step();

    // 3. Verify results are populated in UI
    let _ = harness.get_by_label("Settings");

    // 4. Select and execution
    harness.get_by_label("Settings").click();
    harness.step();
    harness.step();
    harness.step();

    // Palette should be closed
    assert!(!harness.state().app_state_for_test().command_palette.is_open);

    // Settings should be open
    assert!(harness.state().app_state_for_test().layout.show_settings);

    let _ = std::fs::remove_file(&settings_path);
}

#[test]
fn test_integration_command_palette_keyboard_navigation() {
    /* WHY: Verify that arrow keys correctly update the selection index in the palette UI. */
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
        .trigger_action(AppAction::ToggleKatanaCommandPalette);
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
fn test_keyboard_navigation_state() {
    /* WHY: Low-level state check: ensure move_up/move_down wrap around correctly in CommandPaletteState. */
    let mut state = CommandPaletteState {
        results: vec![
            CommandPaletteResult {
                id: "1".into(),
                label: "Item 1".into(),
                secondary_label: None,
                shortcut: None,
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
                shortcut: None,
                score: 0.9,
                kind: CommandPaletteResultKind::Action,
                execute_payload: CommandPaletteExecutePayload::DispatchAppAction(
                    AppAction::ToggleExplorer,
                ),
            },
        ],
        ..Default::default()
    };

    assert_eq!(state.selected_index, 0);

    state.move_down();
    assert_eq!(state.selected_index, 1);

    state.move_down();
    assert_eq!(state.selected_index, 0);

    state.move_up();
    assert_eq!(state.selected_index, 1);

    state.move_up();
    assert_eq!(state.selected_index, 0);
}
