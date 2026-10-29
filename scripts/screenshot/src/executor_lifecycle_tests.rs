use super::{HARNESS_PIXELS_PER_POINT, run_command_palette};
use super::{ObservedFrameProgress, RuntimeSnapshot, assert_runtime_snapshot};
use crate::request::AssertRuntimeSnapshotStep;
use egui_kittest::Harness;
use katana_ui::shell::DocumentResourceSnapshotForTest;
use katana_ui::{app_state::AppState, shell::KatanaApp};
use std::path::PathBuf;
use tempfile::TempDir;

fn command_palette_harness(
    directory: &TempDir,
    document_path: PathBuf,
) -> Harness<'static, KatanaApp> {
    let settings_directory = directory.path().join("config");
    std::fs::create_dir_all(&settings_directory).expect("create isolated settings directory");
    let settings_path = settings_directory.join("settings.json");

    Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_pixels_per_point(HARNESS_PIXELS_PER_POINT)
        .build_eframe(move |cc| {
            use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
            use katana_platform::SettingsService;

            let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
            katana_ui::font_loader::SystemFontLoader::setup_fonts(&cc.egui_ctx, preset, None, None);
            katana_ui::svg_loader::KatanaSvgLoader::install(&cc.egui_ctx);

            let repository = katana_platform::JsonFileRepository::new(settings_path.clone());
            let settings = SettingsService::new(Box::new(repository));
            let icon_pack = settings.settings().theme.icon_pack.clone();
            let icon_settings = settings.settings().icon.clone();
            katana_ui::IconRegistry::install_pack_by_id(&cc.egui_ctx, &icon_pack, &icon_settings);

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
            state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(
                Box::new(katana_platform::workspace::InMemoryWorkspaceRepository::default()),
            );
            state
                .document
                .open_documents
                .push(katana_core::document::Document::new(
                    document_path,
                    String::new(),
                ));
            state.document.active_doc_idx = Some(0);
            let _ = state.config.try_save_settings();

            let mut app = KatanaApp::new(state);
            app.skip_splash();
            app.disable_update_check_for_test();
            app.disable_changelog_popup_for_test();
            app
        })
}

fn idle_snapshot() -> RuntimeSnapshot {
    RuntimeSnapshot {
        rss_kib: 0,
        ui_frame: 0,
        observed: ObservedFrameProgress::default(),
        previews: 0,
        html_surfaces: 0,
        document_surfaces: 0,
        office_workers: 0,
        document_workers: 0,
        kdv_resources: DocumentResourceSnapshotForTest::default(),
        frames: 0,
        textures: 0,
        cache_entries: 0,
    }
}

fn idle_limits() -> AssertRuntimeSnapshotStep {
    AssertRuntimeSnapshotStep {
        baseline: "idle".to_owned(),
        max_rss_delta_kib: 0,
        min_ui_frame_delta: 0,
        min_html_frames_observed: 0,
        min_document_frames_observed: 0,
        max_previews: 0,
        max_html_surfaces: 0,
        max_document_surfaces: 0,
        max_office_workers: 0,
        max_frames: 0,
        max_textures: 0,
        max_cache_entries: 0,
    }
}

#[test]
fn idle_acceptance_rejects_a_worker_after_tab_removal() {
    let idle = idle_snapshot();
    let current = RuntimeSnapshot {
        document_workers: 1,
        ..idle
    };
    let error = assert_runtime_snapshot(&idle, &current, &idle_limits()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("document lifecycle resources remain")
    );
    assert!(assert_runtime_snapshot(&idle, &idle, &idle_limits()).is_ok());
}

#[test]
fn idle_acceptance_rejects_every_published_kdv_resource() {
    let updates: [fn(&mut DocumentResourceSnapshotForTest); 8] = [
        |r| r.live_document_sessions = 1,
        |r| r.live_spreadsheet_workers = 1,
        |r| r.live_worker_workspaces = 1,
        |r| r.retained_artifact_bytes = 1,
        |r| r.cached_page_count = 1,
        |r| r.cached_page_bytes = 1,
        |r| r.cached_spreadsheet_cell_count = 1,
        |r| r.cached_spreadsheet_cell_bytes = 1,
    ];
    let idle = idle_snapshot();
    for update in updates {
        let mut current = idle;
        update(&mut current.kdv_resources);
        let error = assert_runtime_snapshot(&idle, &current, &idle_limits()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("document lifecycle resources remain")
        );
    }
}

#[test]
fn command_palette_real_text_input_respects_active_document_operations() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir().expect("create isolated palette settings");
    let document_path = temporary.path().join("policy.pdf");
    std::fs::write(&document_path, b"%PDF-1.7\n")?;
    let mut harness = command_palette_harness(&temporary, document_path);

    run_command_palette(&mut harness, None, ">", false, false, 0.0, 0.0)?;

    let palette = &harness.state_mut().app_state_mut().command_palette;
    assert!(
        palette.is_open,
        "the real palette UI remains open for inspection"
    );
    assert_eq!(palette.current_query, ">");
    let result_ids: Vec<&str> = palette
        .results
        .iter()
        .map(|result| result.id.as_str())
        .collect();
    assert!(result_ids.contains(&"view.command_palette"));
    for unavailable_id in [
        "view.toggle_split_mode",
        "view.toggle_code_preview",
        "view.toggle_slideshow",
    ] {
        assert!(
            !result_ids.contains(&unavailable_id),
            "PDF operation {unavailable_id} must not appear in the real command palette"
        );
    }

    Ok(())
}
