use egui_kittest::Harness;
// use egui_kittest::kittest::Queryable;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_ui::app_state::AppState;
use katana_ui::shell::KatanaApp;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/* WHY: Unique path generation for concurrent-safe integration tests. */
pub fn unique_temp_path(prefix: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), id))
}

/* WHY: Ensure a fresh, empty directory for each test case to avoid state leakage. */
pub fn fresh_temp_dir(prefix: &str) -> PathBuf {
    let temp_dir = unique_temp_path(prefix);
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    temp_dir
}

/* WHY: Helper to wait for workspace async tasks to complete. */
pub fn wait_for_workspace_load(harness: &mut Harness<'static, KatanaApp>) {
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

/* WHY: Wait until the file tree has at least N entries (useful for recursive loads). */
pub fn wait_for_workspace_tree(harness: &mut Harness<'static, KatanaApp>, min_entries: usize) {
    for attempt in 0..100 {
        harness.step();
        let count = harness
            .state_mut()
            .app_state_mut()
            .workspace
            .data
            .as_ref()
            .map_or(0, |workspace| workspace.tree.len());
        if count >= min_entries {
            break;
        }
        if attempt < 5 {
            std::thread::yield_now();
        } else {
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
}

/* WHY: Centralized harness setup with test-specific settings isolated from user config. */
pub fn setup_harness() -> Harness<'static, KatanaApp> {
    let harness_dir = fresh_temp_dir("katana_test_settings_harness");
    let settings_path = harness_dir.join("settings.json");
    let _ = std::fs::remove_file(&settings_path);

    Harness::builder()
        .with_size(eframe::egui::vec2(1200.0, 800.0))
        .build_eframe(move |_cc| {
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
        let _ = state.config.try_save_settings();

        /* WHY: Use in-memory repository for integration tests to prevent disk pollution and allow temp paths */
        state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
            katana_platform::workspace::InMemoryWorkspaceRepository::default(),
        ));
        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app.disable_update_check_for_test();
        app.disable_changelog_popup_for_test();
        app
    })
}

/* WHY: Specialized harness for testing workspace state persistence logic. */
pub fn setup_harness_with_json_repo(
    settings_path: &std::path::Path,
) -> Harness<'static, KatanaApp> {
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

        /* WHY: Use in-memory repository for integration tests to prevent disk pollution and allow temp paths */
        state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
            katana_platform::workspace::InMemoryWorkspaceRepository::default(),
        ));
        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app.disable_update_check_for_test();
        app
    })
}

/* WHY: Deeply flatten egui shapes which may be nested in Shape::Vec.
 * Essential for verifying low-level rendering output in integration tests. */
pub fn flatten_shapes(shapes: &[egui::epaint::Shape]) -> Vec<egui::epaint::Shape> {
    let mut flattened = Vec::new();
    for shape in shapes {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                flattened.extend(flatten_shapes(shapes));
            }
            _ => {
                flattened.push(shape.clone());
            }
        }
    }
    flattened
}

/* WHY: Helper for egui::FullOutput::shapes which is a Vec<ClippedShape>. */
pub fn flatten_clipped_shapes(
    clipped_shapes: &[egui::epaint::ClippedShape],
) -> Vec<egui::epaint::Shape> {
    let shapes: Vec<_> = clipped_shapes.iter().map(|cs| cs.shape.clone()).collect();
    flatten_shapes(&shapes)
}
