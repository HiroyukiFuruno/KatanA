use egui_kittest::Harness;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_ui::app_state::{AppAction, AppState};
use katana_ui::shell::KatanaApp;
use std::sync::atomic::{AtomicUsize, Ordering};

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), id))
}

fn wait_for_workspace_load(harness: &mut Harness<'static, KatanaApp>) {
    for attempt in 0..100 {
        harness.step();
        if !harness.state_mut().app_state_mut().workspace.is_loading {
            break;
        }
        if attempt < 5 {
            std::thread::yield_now();
        } else {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
}

fn setup_harness() -> Harness<'static, KatanaApp> {
    katana_ui::i18n::set_language("en");
    let settings_path = unique_temp_path("katana_test_i18n_settings").with_extension("json");
    let _ = std::fs::remove_file(&settings_path);
    Harness::builder().build_eframe(move |_cc| {
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let mut state = AppState::new(
            ai_registry,
            plugin_registry,
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(settings_path.clone()),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());
        state
            .config
            .settings
            .settings_mut()
            .updates
            .previous_app_version = Some(katana_ui::about_info::APP_VERSION.to_string());

        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    })
}

fn setup_harness_with_json_repo(settings_path: &std::path::Path) -> Harness<'static, KatanaApp> {
    katana_ui::i18n::set_language("en");
    let path = settings_path.to_path_buf();
    Harness::builder().build_eframe(move |_cc| {
        let repo = katana_platform::JsonFileRepository::new(path.clone());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            settings,
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());
        state
            .config
            .settings
            .settings_mut()
            .updates
            .previous_app_version = Some(katana_ui::about_info::APP_VERSION.to_string());

        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    })
}

#[test]
fn test_persistence_language_roundtrip() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
        harness.step();

        let json = std::fs::read_to_string(&settings_path).unwrap();
        assert!(
            json.contains("\"language\": \"ja\""),
            "settings.json should contain language=ja, got: {json}"
        );
        katana_ui::i18n::set_language("en");
    }

    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        assert_eq!(
            settings.settings().language,
            "ja",
            "Language should be restored as 'ja' from disk"
        );
    }
}

#[test]
fn test_persistence_multiple_changes_accumulate() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    let ws_dir = tempfile::tempdir().unwrap();
    std::fs::write(ws_dir.path().join("readme.md"), "# Readme").unwrap();

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        wait_for_workspace_load(&mut harness);

        harness
            .state_mut()
            .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
        harness.step();
    }

    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let s = settings.settings();

        assert!(
            s.workspace.last_workspace.is_some(),
            "last_workspace should be persisted"
        );
        assert_eq!(s.language, "ja", "language should be persisted");
        katana_ui::i18n::set_language("en");
    }
}

#[test]
fn test_ui_all_languages_load_successfully() {
    let mut harness = setup_harness();
    harness.step();

    let supported_langs = [
        ("en", "English"),
        ("ja", "日本語"),
        ("zh-CN", "简体中文"),
        ("zh-TW", "繁體中文"),
        ("ko", "한국어"),
        ("pt", "Português"),
        ("fr", "Français"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("it", "Italiano"),
    ];

    for (code, _name) in supported_langs {
        harness
            .state_mut()
            .trigger_action(AppAction::ChangeLanguage(code.to_string()));
        harness.step();
        harness.step();

        let settings = katana_ui::i18n::get();
        assert!(
            !settings.settings.tabs.is_empty(),
            "Tabs shouldn't be empty for {}",
            code
        );
        assert_eq!(
            harness
                .state_mut()
                .app_state_mut()
                .config
                .settings
                .settings()
                .language,
            code,
            "Language setting should be updated to {}",
            code
        );
    }
    katana_ui::i18n::set_language("en");
}
