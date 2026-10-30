use crate::shell::KatanaApp;
use egui_kittest::Harness;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[path = "harness_diagnostics.rs"]
mod diagnostics;

const FRAME_TIMEOUT: Duration = Duration::from_secs(15);
pub(in crate::app::doc_close_latency_tests) const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
const VIEWPORT_WIDTH: f32 = 1024.0;
const VIEWPORT_HEIGHT: f32 = 768.0;
const HTML_VIEWPORT_WIDTH: u32 = VIEWPORT_WIDTH as u32;
const HTML_VIEWPORT_HEIGHT: u32 = VIEWPORT_HEIGHT as u32;
const HTML_INITIAL_FRAME_STEPS: usize = 4;
pub(in crate::app::doc_close_latency_tests) const INITIAL_HARNESS_FRAME_STEPS: usize = 4;

const VIEWPORT: eframe::egui::Vec2 = eframe::egui::vec2(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);

pub(in crate::app::doc_close_latency_tests) fn make_harness(
    directory: &TempDir,
) -> Harness<'static, KatanaApp> {
    let settings_directory = directory.path().join("config");
    std::fs::create_dir_all(&settings_directory).expect("create isolated settings directory");
    let settings_path = settings_directory.join("settings.json");
    Harness::builder()
        .with_size(VIEWPORT)
        .with_pixels_per_point(1.0)
        .build_eframe(move |cc| {
            use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
            use katana_platform::SettingsService;

            let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
            crate::font_loader::SystemFontLoader::setup_fonts(&cc.egui_ctx, preset, None, None);
            crate::svg_loader::KatanaSvgLoader::install(&cc.egui_ctx);

            let repository = katana_platform::JsonFileRepository::new(settings_path.clone());
            let settings = SettingsService::new(Box::new(repository));
            let icon_pack = settings.settings().theme.icon_pack.clone();
            let icon_settings = settings.settings().icon.clone();
            crate::IconRegistry::install_pack_by_id(&cc.egui_ctx, &icon_pack, &icon_settings);

            let mut state = crate::app_state::AppState::new(
                AiProviderRegistry::new(),
                PluginRegistry::new(),
                settings,
                std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            );
            state.config.settings.settings_mut().terms_accepted_version =
                Some(crate::about_info::APP_VERSION.to_string());
            state
                .config
                .settings
                .settings_mut()
                .updates
                .previous_app_version = Some(crate::about_info::APP_VERSION.to_string());
            state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(
                Box::new(katana_platform::workspace::InMemoryWorkspaceRepository::default()),
            );
            let _ = state.config.try_save_settings();
            let mut app = KatanaApp::new(state);
            app.skip_splash();
            app.disable_update_check_for_test();
            app.disable_changelog_popup_for_test();
            app
        })
}

pub(in crate::app::doc_close_latency_tests) fn await_html_frame(
    harness: &mut Harness<'_, KatanaApp>,
) -> u64 {
    for _ in 0..HTML_INITIAL_FRAME_STEPS {
        harness.step();
    }
    let viewport = katana_document_viewer::browser_session::HtmlBrowserViewport::new(
        HTML_VIEWPORT_WIDTH,
        HTML_VIEWPORT_HEIGHT,
        1.0,
    )
    .expect("valid HTML browser viewport");
    let already_loaded = harness
        .state_mut()
        .html_browser_frame_generation_for_test()
        .is_some();
    if !already_loaded {
        let app = harness.state_mut();
        let active_path = app.state.active_path().expect("active HTML document");
        let preview = app
            .tab_previews
            .iter_mut()
            .find(|preview| preview.path == active_path)
            .expect("active HTML preview");
        let _ = preview.pane.start_html_browser_for_test(viewport);
    }
    let context = harness.ctx.clone();
    harness
        .state_mut()
        .wait_for_html_browser_frame_for_test(&context, FRAME_TIMEOUT)
        .expect("real HTML browser frame");
    let generation = harness
        .state_mut()
        .html_browser_frame_generation_for_test()
        .expect("HTML adapter must produce a real frame");
    let deadline = Instant::now() + FRAME_TIMEOUT;
    while harness.state_mut().html_browser_is_idle_for_test() != Some(true) {
        assert!(
            Instant::now() < deadline,
            "HTML adapter did not become idle after its frame"
        );
        harness.step();
        thread::yield_now();
    }
    generation
}

pub(in crate::app::doc_close_latency_tests) fn await_office_frame(
    harness: &mut Harness<'_, KatanaApp>,
    expected: &str,
) -> String {
    let deadline = Instant::now() + FRAME_TIMEOUT;
    loop {
        harness.step();
        if let Some(error) = harness.state_mut().document_failure_for_test() {
            panic!("{expected} fixture failed to load through KatanA: {error}");
        }
        if let Some((_, format, active_index, item_count, surface)) =
            harness.state_mut().document_frame_for_test()
            && format == expected
            && item_count > 0
            && harness.state_mut().document_is_idle_for_test() == Some(true)
        {
            return format!(
                "format={format} active_index={active_index} item_count={item_count} surface={surface}"
            );
        }
        assert!(
            Instant::now() < deadline,
            "{expected} real frame deadline exceeded: {}",
            diagnostics::TimeoutDiagnostics::capture(harness)
        );
        thread::yield_now();
    }
}

pub(in crate::app::doc_close_latency_tests) fn await_workers_idle(
    harness: &mut Harness<'_, KatanaApp>,
    baseline: usize,
    format: &str,
    deadline: Instant,
) -> u128 {
    let wait_started = Instant::now();
    assert_worker_deadline(harness, deadline, format);
    while harness
        .state_mut()
        .document_lifecycle_resources_for_test()
        .0
        > baseline
    {
        assert_worker_deadline(harness, deadline, format);
        harness.step();
        assert_worker_deadline(harness, deadline, format);
        thread::yield_now();
    }
    assert_worker_deadline(harness, deadline, format);
    wait_started.elapsed().as_millis()
}

pub(in crate::app::doc_close_latency_tests) fn assert_before_deadline(
    deadline: Instant,
    format: &str,
    phase: &str,
) {
    assert!(
        Instant::now() < deadline,
        "{format} {phase} exceeded the shared close deadline"
    );
}

fn assert_worker_deadline(harness: &mut Harness<'_, KatanaApp>, deadline: Instant, format: &str) {
    assert!(
        Instant::now() < deadline,
        "{format} worker idle exceeded the shared close deadline: lifecycle={:?} host={:?}",
        harness.state().document_lifecycle_resources_for_test(),
        harness.state().preview_resource_counts_for_test()
    );
}
