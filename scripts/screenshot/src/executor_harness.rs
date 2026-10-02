use crate::capture::PngBounds;
use crate::http_fixture::FixtureHttpServer;
use crate::request::{
    AssertActiveDocumentStep, AssertDiffReviewStep, AssertHtmlBrowserFrameContainsRgbStep,
    AssertHtmlBrowserOriginStep, ClickButton, Fixture, ScrollDirection, Step, UiAction,
    VideoFormat,
};
use anyhow::{Context, Result, bail, ensure};
use egui_kittest::{Harness, kittest::Queryable};
use katana_core::markdown::ExporterTrait;
use katana_core::system::ProcessService;
use katana_core::workspace::TreeEntry;
use katana_platform::theme::{ThemeMode, ThemePreset};
use katana_ui::app_state::{AppAction, AppState, SettingsSection, SettingsTab};
use katana_ui::shell::KatanaApp;
use katana_ui::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
};
use katana_ui::state::command_palette_providers::{
    AppCommandProvider, MarkdownContentProvider, WorkspaceFileProvider,
};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;

const HARNESS_PIXELS_PER_POINT: f32 = 2.0;
const DOCUMENT_SCREENSHOT_SETTLE_TIMEOUT_SECONDS: f64 = 30.0;

#[cfg(test)]
#[path = "executor_lifecycle_tests.rs"]
mod lifecycle_tests;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlBrowserFrameIdentity {
    document_path: PathBuf,
    origin: String,
    generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentFrameIdentity {
    document_path: PathBuf,
    format: String,
    active_index: usize,
    item_count: usize,
    node_kind: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct ObservedFrameProgress {
    html: u64,
    document: u64,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeSnapshot {
    rss_kib: u64,
    ui_frame: u64,
    observed: ObservedFrameProgress,
    previews: usize,
    html_surfaces: usize,
    document_surfaces: usize,
    office_workers: usize,
    document_workers: usize,
    kdv_resources: katana_ui::shell::DocumentResourceSnapshotForTest,
    frames: usize,
    textures: usize,
    cache_entries: usize,
}

struct ActiveRecording {
    output_name: String,
    format: VideoFormat,
    fps: u32,
    frame_dir: TempDir,
    next_frame_index: u32,
    frame_time_accumulator: f64,
}

impl ActiveRecording {
    fn new(output_name: String, format: VideoFormat, fps: u32) -> Result<Self> {
        let frame_dir = tempfile::Builder::new()
            .prefix("katana-video-frames-")
            .tempdir()
            .context("failed to create temp frame directory for recording")?;
        Ok(Self {
            output_name,
            format,
            fps: fps.max(1),
            frame_dir,
            next_frame_index: 0,
            frame_time_accumulator: 0.0,
        })
    }

    fn extension(&self) -> &'static str {
        match self.format {
            VideoFormat::Webm => "webm",
            VideoFormat::Mp4 => "mp4",
        }
    }

    fn should_capture_this_tick(&mut self, delta_seconds: f64) -> bool {
        self.frame_time_accumulator += delta_seconds;
        let frame_interval = 1.0 / self.fps.max(1) as f64;
        if self.frame_time_accumulator + f64::EPSILON >= frame_interval {
            self.frame_time_accumulator -= frame_interval;
            true
        } else {
            false
        }
    }

    fn capture_frame(&mut self, harness: &mut Harness<'_, KatanaApp>) -> Result<()> {
        let frame_path = self
            .frame_dir
            .path()
            .join(format!("frame_{:06}.png", self.next_frame_index));
        let image = harness
            .render()
            .map_err(|e| anyhow::anyhow!("render failed during recording: {e}"))?;
        image
            .save(&frame_path)
            .with_context(|| format!("failed to save frame {}", frame_path.display()))?;
        self.next_frame_index += 1;
        Ok(())
    }
}

pub fn run(
    steps: &[Step],
    fixture: &Fixture,
    config_dir: &Path,
    workspace_dir: Option<&Path>,
    output_dir: &Path,
) -> Result<()> {
    let launch = steps.iter().find_map(|s| {
        if let Step::Launch(ls) = s {
            Some(ls)
        } else {
            None
        }
    });
    let (width, height) = match launch {
        Some(launch) => launch.viewport_size()?,
        None => (1728.0, 1117.0),
    };

    ensure!(
        width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0,
        "logical viewport dimensions must be positive and finite"
    );
    let locale = fixture.settings.locale.as_deref().unwrap_or("en");
    katana_ui::i18n::I18nOps::set_language(locale);
    let auto_select_first_file = !steps.iter().any(step_opens_document);

    let settings_path = config_dir.join("settings.json");
    let workspace_dir_owned = workspace_dir.map(|p| p.to_path_buf());
    let workspace_dir_for_lookup = workspace_dir_owned.clone();
    let output_dir = output_dir.to_path_buf();
    let http_server = fixture
        .http_server
        .as_ref()
        .map(|config| {
            let root = workspace_dir.context("HTTP fixture requires a workspace directory")?;
            FixtureHttpServer::start(root, config)
        })
        .transpose()?;

    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .with_pixels_per_point(HARNESS_PIXELS_PER_POINT)
        .build_eframe(move |cc| {
            use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
            use katana_platform::SettingsService;

            let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
            katana_ui::font_loader::SystemFontLoader::setup_fonts(&cc.egui_ctx, preset, None, None);
            katana_ui::svg_loader::KatanaSvgLoader::install(&cc.egui_ctx);

            let repo = katana_platform::JsonFileRepository::new(settings_path.clone());
            let settings = SettingsService::new(Box::new(repo));
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
            let _ = state.config.try_save_settings();
            let mut app = KatanaApp::new(state);
            app.skip_splash();
            app.disable_update_check_for_test();
            app.disable_changelog_popup_for_test();
            if let Some(ref dir) = workspace_dir_owned {
                app.trigger_action(AppAction::OpenWorkspace(dir.clone()));
            }
            app
        });

    for _ in 0..10 {
        harness.step();
    }
    if let Some(show_controls) = fixture.settings.preview_show_diagram_controls {
        harness.ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("katana_preview_diagram_controls"),
                show_controls,
            );
        });
    }
    let mut recording: Option<ActiveRecording> = None;
    let mut observed_frames = ObservedFrameProgress::default();
    let mut runtime_snapshots = HashMap::<String, RuntimeSnapshot>::new();

    for (i, step) in steps.iter().enumerate() {
        let label = match step {
            Step::Launch(_) => "launch",
            Step::Wait(_) => "wait",
            Step::Screenshot(_) => "screenshot",
            Step::AssertScreenshotChanged(_) => "assert_screenshot_changed",
            Step::AssertScreenshotContainsRgb(_) => "assert_screenshot_contains_rgb",
            Step::RecordStart(_) => "record_start",
            Step::RecordStop(_) => "record_stop",
            Step::Scroll(_) => "scroll",
            Step::ExportPng(_) => "export_png",
            Step::OpenFile(_) => "open_file",
            Step::OpenWorkspace(_) => "open_workspace",
            Step::AssertActiveDocument(_) => "assert_active_document",
            Step::AssertDocumentFrame(_) => "assert_document_frame",
            Step::AssertHtmlBrowserOrigin(_) => "assert_html_browser_origin",
            Step::AssertHtmlBrowserFrameContainsRgb(_) => "assert_html_browser_frame_contains_rgb",
            Step::AssertHtmlBrowserViewportMatchesDisplayRect => {
                "assert_html_browser_viewport_matches_display_rect"
            }
            Step::AssertHtmlBrowserDisplayCornersRgb(_) => {
                "assert_html_browser_display_corners_rgb"
            }
            Step::AssertHttpRequests(_) => "assert_http_requests",
            Step::AssertUrlHistory(_) => "assert_url_history",
            Step::AssertDiffReview(_) => "assert_diff_review",
            Step::RecordRuntimeSnapshot(_) => "record_runtime_snapshot",
            Step::RecordPreviewGeometry(_) => "record_preview_geometry",
            Step::RecordTypography(_) => "record_typography",
            Step::AssertRuntimeSnapshot(_) => "assert_runtime_snapshot",
            Step::Action(_) => "action",
            Step::Drag(_) => "drag",
            Step::Quit => "quit",
        };
        println!("step {}/{}: {label}", i + 1, steps.len());
        let operation_started = Instant::now();

        match step {
            Step::Launch(s) => {
                let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                let frames = ((s.wait_seconds * fps) as usize).max(30);
                for _ in 0..frames {
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                }
                for _ in 0..200 {
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    if !harness.state_mut().app_state_mut().workspace.is_loading {
                        break;
                    }
                }
                // Auto-select the first file so editor is populated
                let first = auto_select_first_file
                    .then(|| {
                        harness
                            .state_mut()
                            .app_state_mut()
                            .workspace
                            .data
                            .as_ref()
                            .and_then(|ws| first_file_in_tree(&ws.tree))
                    })
                    .flatten();
                if let Some(path) = first {
                    harness
                        .state_mut()
                        .trigger_action(AppAction::SelectDocument(path));
                    let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                    for _ in 0..fps {
                        harness.step();
                        maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    }
                }
            }
            Step::Wait(s) => {
                // Step egui frames AND sleep real time so async work (network
                // fetches, subprocess launches) actually completes.
                let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                let frames = ((s.seconds * fps) as usize).max(1);
                for _ in 0..frames {
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    sleep_frame(fps);
                }
            }
            Step::Screenshot(s) => {
                harness.run_steps(120);
                wait_for_document_surface_idle(
                    &mut harness,
                    recording.as_mut(),
                    DOCUMENT_SCREENSHOT_SETTLE_TIMEOUT_SECONDS,
                    "screenshot frame",
                )?;
                // 初回renderで確定する実viewportを非同期gridへ反映してから撮影する。
                harness
                    .render()
                    .map_err(|e| anyhow::anyhow!("preflight render failed: {e}"))?;
                wait_for_document_surface_idle(
                    &mut harness,
                    recording.as_mut(),
                    DOCUMENT_SCREENSHOT_SETTLE_TIMEOUT_SECONDS,
                    "screenshot viewport materialization",
                )?;
                let image = harness
                    .render()
                    .map_err(|e| anyhow::anyhow!("render failed: {e}"))?;
                let image = if let Some(crop) = s.crop {
                    image::imageops::crop_imm(&image, crop.x, crop.y, crop.width, crop.height)
                        .to_image()
                } else {
                    image
                };
                let out = output_dir.join(format!("{}.png", s.output_name));
                image
                    .save(&out)
                    .map_err(|e| anyhow::anyhow!("save failed: {e}"))?;
                println!("  saved: {}", out.display());
            }
            Step::AssertScreenshotChanged(assertion) => {
                let baseline = output_dir.join(format!("{}.png", assertion.baseline));
                let current = output_dir.join(format!("{}.png", assertion.current));
                let changed = crate::capture::assert_png_changed(
                    &baseline,
                    &current,
                    assertion.min_changed_pixels,
                )?;
                println!("  changed pixels: {changed}");
            }
            Step::AssertScreenshotContainsRgb(assertion) => {
                let screenshot = output_dir.join(format!("{}.png", assertion.screenshot));
                let matching = crate::capture::assert_png_contains_rgb(
                    &screenshot,
                    assertion.rgb,
                    assertion.tolerance,
                    assertion.min_pixels,
                )?;
                println!("  matching RGB pixels: {matching}");
            }
            Step::RecordStart(s) => {
                if recording.is_some() {
                    bail!("record_start called while another recording is active");
                }
                let format = s.format.unwrap_or(VideoFormat::Webm);
                let fps = s.fps.unwrap_or(24);
                let mut recorder = ActiveRecording::new(s.output_name.clone(), format, fps)?;
                recorder.capture_frame(&mut harness)?;
                println!(
                    "  recording started: {}.{} (fps={})",
                    recorder.output_name,
                    recorder.extension(),
                    recorder.fps
                );
                recording = Some(recorder);
            }
            Step::RecordStop(_) => {
                let mut recorder = recording
                    .take()
                    .context("record_stop called without a matching record_start")?;
                if recorder.next_frame_index == 0 {
                    recorder.capture_frame(&mut harness)?;
                }
                let out =
                    output_dir.join(format!("{}.{}", recorder.output_name, recorder.extension()));
                encode_video(&recorder, &out)?;
                println!("  recorded: {}", out.display());
            }
            Step::Scroll(s) => {
                let scroll_expectation = harness
                    .state_mut()
                    .html_browser_frame_scroll_metrics_for_test()
                    .and_then(|(scroll_y, _)| {
                        let viewport_height = harness
                            .state_mut()
                            .html_browser_frame_viewport_for_test()?
                            .1;
                        let display_height = harness
                            .state_mut()
                            .html_browser_display_rect_for_test()?
                            .height();
                        (display_height > 0.0)
                            .then_some((scroll_y, viewport_height / display_height))
                    });
                let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                let frames = ((s.duration_seconds * fps) as usize).max(1);
                let delta_per_frame = s.pixels / frames as f32;
                let mut delivered_browser_pixels = 0.0;
                for _ in 0..frames {
                    let viewport = harness.ctx.viewport_rect();
                    let pos = egui::pos2(viewport.center().x, viewport.center().y);
                    let delta = scroll_delta(s.direction, delta_per_frame);
                    harness
                        .input_mut()
                        .events
                        .push(egui::Event::PointerMoved(pos));
                    harness.input_mut().events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta,
                        modifiers: egui::Modifiers::NONE,
                        phase: egui::TouchPhase::Move,
                    });
                    harness.step();
                    if let Some((_, browser_scale)) = scroll_expectation {
                        let smooth_y = harness.ctx.input(|input| input.smooth_scroll_delta.y);
                        delivered_browser_pixels += (-smooth_y * browser_scale).abs();
                    }
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    sleep_frame(fps);
                }
                let viewport = harness.ctx.viewport_rect();
                let pos = egui::pos2(viewport.center().x, viewport.center().y);
                harness
                    .input_mut()
                    .events
                    .push(egui::Event::PointerMoved(pos));
                harness.input_mut().events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::Vec2::ZERO,
                    modifiers: egui::Modifiers::NONE,
                    phase: egui::TouchPhase::End,
                });
                for _ in 0..6 {
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    sleep_frame(fps);
                }
                if matches!(s.direction, ScrollDirection::Up | ScrollDirection::Down) {
                    if let Some((initial_scroll, _)) = scroll_expectation {
                        wait_for_browser_scroll(
                            &mut harness,
                            recording.as_mut(),
                            initial_scroll,
                            s.direction,
                            delivered_browser_pixels,
                        )?;
                    }
                }
            }
            Step::ExportPng(s) => {
                // Get the active document's markdown content and path
                let doc_info = harness
                    .state_mut()
                    .app_state_mut()
                    .active_document()
                    .map(|d| (d.buffer.clone(), d.path.clone()));
                let (source, doc_path) =
                    doc_info.context("export_png requires an active document")?;
                let preset =
                    katana_core::markdown::color_preset::DiagramColorPreset::current().clone();
                let out = output_dir.join(format!("{}.png", s.output_name));
                let export_input = katana_core::markdown::export::ExportInput {
                    format: katana_core::markdown::export::ExportFormat::Png,
                    markdown_source: source,
                    source_path: doc_path,
                    output_path: out.clone(),
                    config: katana_core::markdown::export::ExportConfig::from_preset(&preset),
                };
                katana_core::markdown::export::ImageExporter
                    .export(&export_input)
                    .map_err(|e| anyhow::anyhow!("png export failed: {e}"))?;
                println!("  exported: {}", out.display());
            }
            Step::OpenFile(s) => {
                let path = harness
                    .state_mut()
                    .app_state_mut()
                    .workspace
                    .data
                    .as_ref()
                    .and_then(|ws| {
                        find_workspace_file(
                            &ws.tree,
                            workspace_dir_for_lookup.as_deref(),
                            &s.file_name,
                        )
                    });
                match path {
                    Some(p) => {
                        let previous_frame = html_browser_frame_identity(&mut harness);
                        let previous_document_frame = document_frame_identity(&mut harness);
                        let previous_document_failure =
                            harness.state_mut().document_failure_for_test();
                        harness
                            .state_mut()
                            .trigger_action(AppAction::SelectDocument(p));
                        ensure!(
                            !(s.wait_for_html_frame && s.wait_for_document_frame),
                            "open_file cannot wait for both HTML and document frames"
                        );
                        if s.wait_for_html_frame {
                            let elapsed = wait_for_html_browser_frame(
                                &mut harness,
                                recording.as_mut(),
                                previous_frame,
                                s.wait_seconds,
                            )?;
                            assert_first_frame_latency(
                                "HTML browser",
                                elapsed,
                                s.max_first_frame_seconds,
                            )?;
                            observed_frames.html = observed_frames.html.saturating_add(1);
                        } else if s.wait_for_document_frame {
                            let (elapsed, frame) = wait_for_document_frame(
                                &mut harness,
                                recording.as_mut(),
                                previous_document_frame,
                                previous_document_failure,
                                s.wait_seconds,
                            )?;
                            assert_first_frame_latency(
                                "document",
                                elapsed,
                                s.max_first_frame_seconds,
                            )?;
                            assert_document_frame(
                                s.expected_document_format.as_deref(),
                                s.expected_document_item_count,
                                s.expected_document_node_kind.as_deref(),
                                &frame,
                            )?;
                            observed_frames.document = observed_frames.document.saturating_add(1);
                        } else {
                            let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                            let frames = ((s.wait_seconds * fps) as usize).max(30);
                            for _ in 0..frames {
                                harness.step();
                                maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                            }
                        }
                    }
                    None => {
                        bail!("file {:?} not found in workspace tree", s.file_name);
                    }
                }
            }
            Step::OpenWorkspace(s) => {
                let path = PathBuf::from(&s.path)
                    .canonicalize()
                    .with_context(|| format!("workspace path not found: {}", s.path))?;
                harness
                    .state_mut()
                    .trigger_action(AppAction::OpenWorkspace(path));
                let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                let frames = ((s.wait_seconds * fps) as usize).max(30);
                for _ in 0..frames {
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                }
                for _ in 0..200 {
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    if !harness.state_mut().app_state_mut().workspace.is_loading {
                        break;
                    }
                }
            }
            Step::AssertActiveDocument(s) => {
                assert_active_document(&mut harness, s)?;
            }
            Step::AssertDocumentFrame(s) => {
                let deadline = async_assert_deadline(s.timeout_seconds)?;
                loop {
                    let frame = document_frame_identity(&mut harness);
                    if frame
                        .as_ref()
                        .is_some_and(|frame| frame.active_index == s.active_index)
                    {
                        break;
                    }
                    ensure!(
                        Instant::now() < deadline,
                        "document frame did not reach active index {}: {:?}",
                        s.active_index,
                        frame
                    );
                    ensure!(
                        harness.state_mut().document_failure_for_test().is_none(),
                        "document failed while waiting for selected sheet"
                    );
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    sleep_frame(60.0);
                }
                let frame = document_frame_identity(&mut harness)
                    .context("expected an active document frame")?;
                assert_document_frame(
                    Some(&s.format),
                    Some(s.item_count),
                    Some(&s.node_kind),
                    &frame,
                )?;
                ensure!(
                    frame.active_index == s.active_index,
                    "document active index mismatch: expected {}, got {}",
                    s.active_index,
                    frame.active_index
                );
            }
            Step::AssertHtmlBrowserOrigin(s) => {
                assert_html_browser_origin(&mut harness, recording.as_mut(), s)?;
            }
            Step::AssertHtmlBrowserFrameContainsRgb(s) => {
                assert_html_browser_frame_contains_rgb(&mut harness, recording.as_mut(), s)?;
            }
            Step::AssertHtmlBrowserViewportMatchesDisplayRect => {
                assert_html_browser_viewport_matches_display_rect(&mut harness)?;
            }
            Step::AssertHtmlBrowserDisplayCornersRgb(s) => {
                let image = harness.render().map_err(|error| {
                    anyhow::anyhow!("render failed before corner assertion: {error}")
                })?;
                let display = harness
                    .state_mut()
                    .html_browser_display_rect_for_test()
                    .context("expected an active HTML display rect")?;
                let bounds = physical_png_bounds(
                    display,
                    image.width(),
                    image.height(),
                    HARNESS_PIXELS_PER_POINT,
                )
                .context("HTML display rect is outside the composed screenshot")?;
                let corners = [
                    (bounds.x, bounds.y),
                    (bounds.x + bounds.width - 1, bounds.y),
                    (bounds.x, bounds.y + bounds.height - 1),
                    (bounds.x + bounds.width - 1, bounds.y + bounds.height - 1),
                ];
                for (x, y) in corners {
                    let pixel = image.get_pixel(x, y).0;
                    ensure!(
                        pixel[..3]
                            .iter()
                            .zip(s.rgb)
                            .all(|(actual, expected)| actual.abs_diff(expected) <= s.tolerance),
                        "HTML display corner ({x}, {y}) is rgb({},{},{}), expected {:?} +/- {}",
                        pixel[0],
                        pixel[1],
                        pixel[2],
                        s.rgb,
                        s.tolerance
                    );
                }
                println!("  HTML display corners matched page RGB: {:?}", s.rgb);
            }
            Step::AssertHttpRequests(s) => {
                let server = http_server
                    .as_ref()
                    .context("assert_http_requests requires fixture.http_server")?;
                assert_http_requests(&mut harness, recording.as_mut(), server, s)?;
            }
            Step::AssertUrlHistory(s) => {
                let history = harness
                    .state_mut()
                    .app_state_for_test()
                    .url_tab
                    .history
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                for suffix in &s.origin_suffixes {
                    ensure!(
                        history.iter().any(|url| url.ends_with(suffix)),
                        "URL history is missing suffix {suffix:?}; recorded {history:?}"
                    );
                }
                println!("  URL history matched: {:?}", s.origin_suffixes);
            }
            Step::AssertDiffReview(s) => {
                assert_diff_review(&mut harness, s)?;
            }
            Step::RecordRuntimeSnapshot(s) => {
                ensure!(
                    !s.name.is_empty(),
                    "runtime snapshot name must not be empty"
                );
                let snapshot = capture_runtime_snapshot(&mut harness, observed_frames)?;
                println!("  runtime snapshot {:?}: {snapshot:?}", s.name);
                runtime_snapshots.insert(s.name.clone(), snapshot);
            }
            Step::RecordPreviewGeometry(s) => {
                ensure!(
                    !s.output_name.is_empty()
                        && Path::new(&s.output_name).components().count() == 1
                        && matches!(
                            Path::new(&s.output_name).components().next(),
                            Some(Component::Normal(_))
                        ),
                    "preview geometry output_name must be a single file stem"
                );
                harness.run();
                wait_for_document_surface_idle(
                    &mut harness,
                    recording.as_mut(),
                    DOCUMENT_SCREENSHOT_SETTLE_TIMEOUT_SECONDS,
                    "preview geometry frame",
                )?;
                katana_ui::preview_pane::screenshot_test_hooks::PreviewOverlayInspectionOps::reset(
                    &harness.ctx,
                );
                // `render()` only rasterizes the latest output, so run exactly one explicit UI pass
                // and capture its overlay counter and state as the same frame.
                harness.step();
                let ui_pass_frame_nr = harness.ctx.cumulative_frame_nr();
                katana_ui::preview_pane::screenshot_test_hooks::PreviewOverlayInspectionOps::complete(
                    &harness.ctx,
                    ui_pass_frame_nr,
                );
                let image = harness
                    .render()
                    .map_err(|error| anyhow::anyhow!("preview geometry render failed: {error}"))?;
                if let Some(output_name) = &s.full_screenshot_output_name {
                    ensure!(
                        !output_name.is_empty()
                            && Path::new(output_name).components().count() == 1
                            && matches!(
                                Path::new(output_name).components().next(),
                                Some(Component::Normal(_))
                            ),
                        "preview geometry full_screenshot_output_name must be a single file stem"
                    );
                    let path = output_dir.join(format!("{output_name}.png"));
                    image.save(&path).with_context(|| {
                        format!(
                            "failed to write preview geometry screenshot {}",
                            path.display()
                        )
                    })?;
                    println!("  preview geometry screenshot: {}", path.display());
                }
                let (viewport, scroll_y, content_top_y) = harness
                    .state()
                    .preview_geometry_for_test()
                    .context("preview geometry requires rendered Markdown geometry")?;
                let physical = physical_png_bounds(
                    viewport,
                    image.width(),
                    image.height(),
                    HARNESS_PIXELS_PER_POINT,
                )
                .context("preview geometry is outside the screenshot")?;
                let expected_physical_width =
                    (s.expected_content_viewport.width as f32 * HARNESS_PIXELS_PER_POINT) as u32;
                let expected_physical_height =
                    (s.expected_content_viewport.height as f32 * HARNESS_PIXELS_PER_POINT) as u32;
                ensure!(
                    physical.width == expected_physical_width
                        && physical.height == expected_physical_height,
                    "preview geometry physical crop is clipped or scaled: actual={}x{}, expected={}x{}",
                    physical.width,
                    physical.height,
                    expected_physical_width,
                    expected_physical_height
                );
                let pointer_position = harness.ctx.input(|input| input.pointer.hover_pos());
                let app_state = harness.state().app_state_for_test();
                let active_editor_line = app_state.scroll.active_editor_line;
                let hovered_preview_line_count = app_state.scroll.hovered_preview_lines.len();
                let overlays = katana_ui::preview_pane::screenshot_test_hooks::PreviewOverlayInspectionOps::snapshot(&harness.ctx)
                    .context("preview overlay inspection was not enabled for the captured UI pass")?;
                let pointer_inside_content =
                    pointer_position.is_some_and(|position| viewport.contains(position));
                let interaction_state = serde_json::json!({
                    "pointer_position": pointer_position.map(|position| serde_json::json!({ "x": position.x, "y": position.y })),
                    "pointer_inside_content": pointer_inside_content,
                    "active_editor_line": active_editor_line,
                    "hovered_preview_line_count": hovered_preview_line_count,
                    "diagram_control_renders": overlays.diagram_control_renders,
                    "image_control_renders": overlays.image_control_renders,
                    "code_copy_control_renders": overlays.code_copy_control_renders,
                    "active_markdown_ranges": overlays.active_markdown_ranges,
                    "hovered_markdown_spans": overlays.hovered_markdown_spans,
                    "image_hover_background_renders": overlays.image_hover_background_renders,
                    "local_image_hover_background_renders": overlays.local_image_hover_background_renders,
                    "code_selection_renders": overlays.code_selection_renders
                    ,"markdown_section_renders": overlays.markdown_section_renders
                    ,"completed_ui_frame_nr": overlays.completed_ui_frame_nr
                });
                if s.require_clean_interaction_state {
                    ensure!(
                        !pointer_inside_content
                            && active_editor_line.is_none()
                            && hovered_preview_line_count == 0
                            && overlays.diagram_control_renders == 0
                            && overlays.image_control_renders == 0
                            && overlays.code_copy_control_renders == 0
                            && overlays.active_markdown_ranges == 0
                            && overlays.hovered_markdown_spans == 0
                            && overlays.image_hover_background_renders == 0
                            && overlays.local_image_hover_background_renders == 0
                            && overlays.code_selection_renders == 0
                            && overlays.markdown_section_renders > 0
                            && overlays.completed_ui_frame_nr == Some(ui_pass_frame_nr),
                        "preview geometry capture contains interaction overlay state: {interaction_state}"
                    );
                }
                let snapshot = serde_json::json!({
                    "schema_version": 1,
                    "pixels_per_point": HARNESS_PIXELS_PER_POINT,
                    "configured_font_size": harness.state().app_state_for_test().config.settings.settings().font.size,
                    "require_clean_interaction_state": s.require_clean_interaction_state,
                    "ui_pass_frame_nr": ui_pass_frame_nr,
                    "interaction_state": interaction_state,
                    "preview_geometry": {
                        "viewport": { "x": viewport.left(), "y": viewport.top(), "width": viewport.width(), "height": viewport.height() },
                        "scroll_y": scroll_y,
                        "content_top_y": content_top_y,
                        "physical_crop": { "x": physical.x, "y": physical.y, "width": physical.width, "height": physical.height },
                        "full_screenshot": {
                            "output_name": s.full_screenshot_output_name,
                            "width": image.width(),
                            "height": image.height()
                        }
                    }
                });
                let path = output_dir.join(format!("{}.json", s.output_name));
                std::fs::write(&path, serde_json::to_vec_pretty(&snapshot)?).with_context(
                    || {
                        format!(
                            "failed to write preview geometry snapshot {}",
                            path.display()
                        )
                    },
                )?;
                println!("  preview geometry snapshot: {}", path.display());
                ensure!(
                    (viewport.width() - s.expected_content_viewport.width as f32).abs() <= 0.01
                        && (viewport.height() - s.expected_content_viewport.height as f32).abs()
                            <= 0.01,
                    "preview geometry content viewport mismatch: actual={:?}, expected={}x{}",
                    viewport.size(),
                    s.expected_content_viewport.width,
                    s.expected_content_viewport.height
                );
                ensure!(
                    (scroll_y - s.expected_scroll_y).abs() <= 0.01,
                    "preview geometry scroll mismatch: actual={scroll_y}, expected={}",
                    s.expected_scroll_y
                );
            }
            Step::RecordTypography(s) => {
                ensure!(
                    !s.output_name.is_empty()
                        && Path::new(&s.output_name).components().count() == 1
                        && matches!(
                            Path::new(&s.output_name).components().next(),
                            Some(Component::Normal(_))
                        ),
                    "typography output_name must be a single file stem"
                );
                harness.run();
                wait_for_document_surface_idle(
                    &mut harness,
                    recording.as_mut(),
                    DOCUMENT_SCREENSHOT_SETTLE_TIMEOUT_SECONDS,
                    "typography frame",
                )?;
                harness
                    .render()
                    .map_err(|error| anyhow::anyhow!("typography render failed: {error}"))?;
                let mut snapshot = katana_paint_metrics::capture(harness.output());
                let texts = snapshot["text_shapes"]
                    .as_array()
                    .context("missing painted text shapes")?;
                for required in &s.required_text {
                    ensure!(
                        texts.iter().any(|text| text["text"]
                            .as_str()
                            .is_some_and(|text| text.contains(required))),
                        "required typography text was not painted: {required:?}"
                    );
                }
                let anchors = harness
                    .state()
                    .preview_anchors_for_test()
                    .context("typography capture requires an active preview")?;
                snapshot["source_anchors"] = serde_json::json!(anchors.iter().map(|(kind, lines, rect)| {
                    serde_json::json!({
                        "kind": kind,
                        "source_anchor_range_zero_based": [lines.start, lines.end],
                        "rect": { "x": rect.left(), "y": rect.top(), "width": rect.width(), "height": rect.height() }
                    })
                }).collect::<Vec<_>>());
                snapshot["configured_font_size"] = serde_json::json!(
                    harness
                        .state()
                        .app_state_for_test()
                        .config
                        .settings
                        .settings()
                        .font
                        .size
                );
                snapshot["schema_version"] = serde_json::json!(1);
                snapshot["accessibility_label_rects"] = serde_json::json!(s.required_text.iter().map(|label| {
                    let rects = harness.query_all_by_label(label).map(|node| {
                        // egui_kittest::Node::rect is already expressed in logical points.
                        let rect = node.rect();
                        serde_json::json!({ "x": rect.left(), "y": rect.top(), "width": rect.width(), "height": rect.height() })
                    }).collect::<Vec<_>>();
                    serde_json::json!({ "label": label, "rects": rects })
                }).collect::<Vec<_>>());
                let (viewport, scroll_y, content_top_y) = harness
                    .state()
                    .preview_geometry_for_test()
                    .context("typography requires rendered Markdown geometry")?;
                snapshot["preview_geometry"] = serde_json::json!({
                    "viewport": { "x": viewport.left(), "y": viewport.top(), "width": viewport.width(), "height": viewport.height() },
                    "scroll_y": scroll_y,
                    "content_top_y": content_top_y
                });
                let path = output_dir.join(format!("{}.json", s.output_name));
                std::fs::write(&path, serde_json::to_vec_pretty(&snapshot)?).with_context(
                    || format!("failed to write typography snapshot {}", path.display()),
                )?;
                println!("  typography snapshot: {}", path.display());
                ensure!(
                    (viewport.width() - s.expected_content_viewport.width as f32).abs() <= 0.01
                        && (viewport.height() - s.expected_content_viewport.height as f32).abs()
                            <= 0.01,
                    "typography content viewport mismatch: actual={:?}, expected={}x{}",
                    viewport.size(),
                    s.expected_content_viewport.width,
                    s.expected_content_viewport.height
                );
                ensure!(
                    (scroll_y - s.expected_scroll_y).abs() <= 0.01,
                    "typography scroll mismatch: actual={scroll_y}, expected={}",
                    s.expected_scroll_y
                );
            }
            Step::AssertRuntimeSnapshot(s) => {
                let baseline = runtime_snapshots.get(&s.baseline).with_context(|| {
                    format!("runtime snapshot {:?} was not recorded", s.baseline)
                })?;
                let current = capture_runtime_snapshot(&mut harness, observed_frames)?;
                assert_runtime_snapshot(baseline, &current, s)?;
                println!(
                    "  runtime snapshot matched {:?}: current={current:?}",
                    s.baseline
                );
            }
            Step::Action(a) => {
                match &a.action {
                    UiAction::OpenUrl {
                        url,
                        timeout_seconds,
                        expected_error_contains,
                    } => {
                        open_url_and_wait_for_html_frame(
                            &mut harness,
                            recording.as_mut(),
                            url,
                            *timeout_seconds,
                            expected_error_contains.as_deref(),
                        )?;
                    }
                    UiAction::OpenFixtureUrl {
                        path,
                        wait_seconds,
                        expected_error_contains,
                    } => {
                        let server = http_server
                            .as_ref()
                            .context("open_fixture_url requires fixture.http_server")?;
                        let url = server.url(path)?;
                        if let Some(expected) = expected_error_contains {
                            open_url_and_wait_for_html_frame(
                                &mut harness,
                                recording.as_mut(),
                                &url,
                                *wait_seconds,
                                Some(expected),
                            )?;
                            continue;
                        }
                        println!(
                            "  queue fixture URL {url}; pending before queue: {:?}",
                            harness.state_mut().pending_action_for_test()
                        );
                        harness.state_mut().trigger_action(AppAction::OpenUrl(url));
                        harness.ctx.request_repaint();
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                        let app = harness.state_mut();
                        println!(
                            "  URL action state: pending={:?}, loading={}, input={:?}, error={:?}",
                            app.pending_action_for_test(),
                            app.app_state_for_test().url_tab.is_loading,
                            app.app_state_for_test().url_tab.input,
                            app.app_state_for_test().url_tab.last_error,
                        );
                    }
                    UiAction::OpenFixtureDocumentUrl {
                        path,
                        timeout_seconds,
                        expected_document_format,
                        expected_document_item_count,
                        expected_document_node_kind,
                    } => {
                        let server = http_server
                            .as_ref()
                            .context("open_fixture_document_url requires fixture.http_server")?;
                        let url = server.url(path)?;
                        let previous = document_frame_identity(&mut harness);
                        let previous_failure = harness.state_mut().document_failure_for_test();
                        harness.state_mut().trigger_action(AppAction::OpenUrl(url));
                        harness.ctx.request_repaint();
                        let (_, frame) = wait_for_document_frame(
                            &mut harness,
                            recording.as_mut(),
                            previous,
                            previous_failure,
                            *timeout_seconds,
                        )?;
                        assert_document_frame(
                            Some(expected_document_format),
                            Some(*expected_document_item_count),
                            Some(expected_document_node_kind),
                            &frame,
                        )?;
                    }
                    UiAction::OpenFixtureDocumentErrorUrl {
                        path,
                        timeout_seconds,
                        expected_error_contains,
                    } => {
                        let server = http_server.as_ref().context(
                            "open_fixture_document_error_url requires fixture.http_server",
                        )?;
                        let url = server.url(path)?;
                        let previous_failure = harness.state_mut().document_failure_for_test();
                        harness.state_mut().trigger_action(AppAction::OpenUrl(url));
                        harness.ctx.request_repaint();
                        wait_for_document_failure(
                            &mut harness,
                            recording.as_mut(),
                            previous_failure.as_deref(),
                            expected_error_contains,
                            *timeout_seconds,
                        )?;
                    }
                    UiAction::OpenSettingsTab { tab } => {
                        if !harness.state_mut().app_state_mut().layout.show_settings {
                            harness
                                .state_mut()
                                .trigger_action(AppAction::ToggleSettings);
                        }
                        for _ in 0..30 {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                        let (settings_tab, settings_section) = parse_settings_tab(tab);
                        let config = &mut harness.state_mut().app_state_mut().config;
                        config.active_settings_tab = settings_tab;
                        config.active_settings_section = settings_section;
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::ForceOpenAccordion { id } => {
                        let egui_id = egui::Id::new(id.as_str());
                        let mut state =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                &harness.ctx,
                                egui_id,
                                false,
                            );
                        state.set_open(true);
                        state.store(&harness.ctx);
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::OpenIconsAdvancedPanel => {
                        harness.ctx.data_mut(|d| {
                            d.insert_temp(egui::Id::new("icons_advanced_is_open"), true);
                        });
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::ScrollDown { amount } => {
                        // Move pointer to center of settings content area, then scroll
                        let viewport = harness.ctx.viewport_rect();
                        let pos = egui::pos2(viewport.center().x, viewport.center().y);
                        harness
                            .input_mut()
                            .events
                            .push(egui::Event::PointerMoved(pos));
                        harness.input_mut().events.push(egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::Vec2::new(0.0, *amount),
                            modifiers: egui::Modifiers::NONE,
                            phase: egui::TouchPhase::Move,
                        });
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::SetScrollOffset { id: _, y } => {
                        // Send wheel events in batches to reach the target offset.
                        // Each batch moves the pointer to center and scrolls negatively
                        // (positive y = scroll up in egui's convention).
                        const BATCH: f32 = 200.0;
                        let mut remaining = *y;
                        while remaining > 0.0 {
                            let delta = remaining.min(BATCH);
                            remaining -= delta;
                            let viewport = harness.ctx.viewport_rect();
                            let pos = egui::pos2(viewport.center().x, viewport.center().y);
                            harness
                                .input_mut()
                                .events
                                .push(egui::Event::PointerMoved(pos));
                            harness.input_mut().events.push(egui::Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::Vec2::new(0.0, -delta),
                                modifiers: egui::Modifiers::NONE,
                                phase: egui::TouchPhase::Move,
                            });
                            for _ in 0..30 {
                                harness.step();
                                maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                            }
                        }
                    }
                    UiAction::OpenFirstChangelogSection => {
                        // Changelog accordion IDs are the version strings.
                        // The first (top) section matches the current app version.
                        let version = katana_ui::about_info::APP_VERSION;
                        let egui_id = egui::Id::new(version);
                        let mut state =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                &harness.ctx,
                                egui_id,
                                false,
                            );
                        state.set_open(true);
                        state.store(&harness.ctx);
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::SetViewMode { mode } => {
                        use katana_ui::app_state::ViewMode;
                        let view_mode = match mode.as_str() {
                            "preview_only" => ViewMode::PreviewOnly,
                            "code_only" => ViewMode::CodeOnly,
                            "split" => ViewMode::Split,
                            other => {
                                println!(
                                    "  WARNING: unknown view mode {other:?}, defaulting to preview_only"
                                );
                                ViewMode::PreviewOnly
                            }
                        };
                        harness
                            .state_mut()
                            .trigger_action(AppAction::SetViewMode(view_mode));
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::RunCommandPalette {
                        query,
                        katana_mode,
                        execute_first,
                        keystroke_delay_seconds,
                        pause_after_seconds,
                    } => {
                        run_command_palette(
                            &mut harness,
                            recording.as_mut(),
                            query,
                            *katana_mode,
                            *execute_first,
                            keystroke_delay_seconds.unwrap_or(0.08),
                            pause_after_seconds.unwrap_or(0.45),
                        )?;
                    }
                    UiAction::RunGlobalSearch {
                        query,
                        tab,
                        keystroke_delay_seconds,
                        pause_after_seconds,
                    } => {
                        run_global_search(
                            &mut harness,
                            recording.as_mut(),
                            query,
                            tab,
                            keystroke_delay_seconds.unwrap_or(0.06),
                            pause_after_seconds.unwrap_or(0.6),
                        )?;
                    }
                    UiAction::RunDocumentSearch {
                        query,
                        next_count,
                        keystroke_delay_seconds,
                        pause_after_seconds,
                    } => {
                        run_document_search(
                            &mut harness,
                            recording.as_mut(),
                            query,
                            next_count.unwrap_or(0),
                            keystroke_delay_seconds.unwrap_or(0.06),
                            pause_after_seconds.unwrap_or(0.5),
                        )?;
                    }
                    UiAction::SelectThemePresetInSettings { preset } => {
                        select_theme_preset_in_settings(&mut harness, recording.as_mut(), preset)?;
                    }
                    UiAction::SlideshowNavigate {
                        direction,
                        steps,
                        wait_seconds,
                    } => {
                        navigate_slideshow(
                            &mut harness,
                            recording.as_mut(),
                            direction,
                            *steps,
                            *wait_seconds,
                        )?;
                    }
                    UiAction::SelectDemoTab { file_name } => {
                        let path = PathBuf::from(format!("Katana://Demo/{file_name}"));
                        let is_open = harness
                            .state_mut()
                            .app_state_mut()
                            .document
                            .open_documents
                            .iter()
                            .any(|doc| doc.path == path);
                        if !is_open {
                            bail!(
                                "demo tab {:?} is not open; call open_help_demo before select_demo_tab",
                                file_name
                            );
                        }
                        harness
                            .state_mut()
                            .trigger_action(AppAction::SelectDocument(path));
                        step_for_seconds(&mut harness, recording.as_mut(), 1.0)?;
                    }
                    UiAction::OpenProblemsPanel => {
                        harness
                            .state_mut()
                            .app_state_mut()
                            .diagnostics
                            .is_panel_open = true;
                        step_for_seconds(&mut harness, recording.as_mut(), 1.0)?;
                    }
                    UiAction::CloseSearchModal => {
                        harness.state_mut().app_state_mut().layout.show_search_modal = false;
                        step_for_seconds(&mut harness, recording.as_mut(), 0.5)?;
                    }
                    UiAction::CloseDocSearch => {
                        harness.state_mut().app_state_mut().search.doc_search_open = false;
                        step_for_seconds(&mut harness, recording.as_mut(), 0.5)?;
                    }
                    UiAction::CloseActiveDocument { wait_seconds } => {
                        close_active_document_and_wait_for_idle(
                            &mut harness,
                            recording.as_mut(),
                            *wait_seconds,
                            "close_active_document",
                        )?;
                    }
                    UiAction::CloseAllDocuments { wait_seconds } => {
                        close_all_documents_and_wait_for_idle(
                            &mut harness,
                            recording.as_mut(),
                            *wait_seconds,
                        )?;
                    }
                    UiAction::RunMixedDocumentCycles {
                        html_file_name,
                        document_file_name,
                        cycles,
                        html_timeout_seconds,
                        document_timeout_seconds,
                        close_timeout_seconds,
                        expected_document_format,
                        expected_document_item_count,
                        expected_document_node_kind,
                    } => {
                        ensure!(*cycles > 0, "mixed document cycle count must be positive");
                        let (html_path, document_path) = {
                            let state = harness.state_mut().app_state_mut();
                            let workspace = state
                                .workspace
                                .data
                                .as_ref()
                                .context("mixed document cycles require an open workspace")?;
                            let html_path = find_workspace_file(
                                &workspace.tree,
                                workspace_dir_for_lookup.as_deref(),
                                html_file_name,
                            )
                            .with_context(|| {
                                format!("HTML cycle file {html_file_name:?} was not found")
                            })?;
                            let document_path = find_workspace_file(
                                &workspace.tree,
                                workspace_dir_for_lookup.as_deref(),
                                document_file_name,
                            )
                            .with_context(|| {
                                format!("document cycle file {document_file_name:?} was not found")
                            })?;
                            (html_path, document_path)
                        };
                        for cycle in 1..=*cycles {
                            let previous_html = html_browser_frame_identity(&mut harness);
                            harness
                                .state_mut()
                                .trigger_action(AppAction::SelectDocument(html_path.clone()));
                            wait_for_html_browser_frame(
                                &mut harness,
                                recording.as_mut(),
                                previous_html,
                                *html_timeout_seconds,
                            )?;
                            observed_frames.html = observed_frames.html.saturating_add(1);
                            close_active_document_and_wait_for_idle(
                                &mut harness,
                                recording.as_mut(),
                                *close_timeout_seconds,
                                &format!("mixed cycle {cycle} HTML close"),
                            )?;

                            let previous_document = document_frame_identity(&mut harness);
                            let previous_failure = harness.state_mut().document_failure_for_test();
                            harness
                                .state_mut()
                                .trigger_action(AppAction::SelectDocument(document_path.clone()));
                            let (_, frame) = wait_for_document_frame(
                                &mut harness,
                                recording.as_mut(),
                                previous_document,
                                previous_failure,
                                *document_timeout_seconds,
                            )?;
                            assert_document_frame(
                                Some(expected_document_format),
                                Some(*expected_document_item_count),
                                Some(expected_document_node_kind),
                                &frame,
                            )?;
                            observed_frames.document = observed_frames.document.saturating_add(1);
                            close_active_document_and_wait_for_idle(
                                &mut harness,
                                recording.as_mut(),
                                *close_timeout_seconds,
                                &format!("mixed cycle {cycle} document close"),
                            )?;
                            println!("  mixed document cycle {cycle}/{cycles} completed");
                        }
                    }
                    UiAction::RefreshDiagnostics => {
                        harness
                            .state_mut()
                            .trigger_action(AppAction::RefreshDiagnostics);
                        step_for_seconds(&mut harness, recording.as_mut(), 1.0)?;
                    }
                    UiAction::DocumentNext { timeout_seconds } => {
                        advance_document_and_wait(
                            &mut harness,
                            recording.as_mut(),
                            *timeout_seconds,
                        )?;
                    }
                    UiAction::ApplyLintFixesForActiveFile => {
                        apply_lint_fixes_for_active_file(&mut harness, recording.as_mut())?;
                    }
                    UiAction::ClickNode {
                        label,
                        button,
                        wait_seconds,
                        expected_bounds,
                    } => {
                        if let Some(bounds) = expected_bounds {
                            let rect = harness.get_by_label(label).rect();
                            assert_node_bounds(rect, harness.ctx.viewport_rect(), *bounds)?;
                        }
                        click_node(&mut harness, label, *button);
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    UiAction::HoverAt { x, y, wait_seconds } => {
                        harness.hover_at(egui::pos2(*x, *y));
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    UiAction::ClickAt {
                        x,
                        y,
                        button,
                        wait_seconds,
                        wait_for_html_frame,
                    } => {
                        if *wait_for_html_frame {
                            click_at_and_wait_for_html_frame(
                                &mut harness,
                                recording.as_mut(),
                                egui::pos2(*x, *y),
                                *button,
                                *wait_seconds,
                            )?;
                        } else {
                            click_at(&mut harness, egui::pos2(*x, *y), *button);
                            step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                        }
                    }
                    UiAction::ClickHtmlViewportFraction {
                        x_fraction,
                        y_fraction,
                        button,
                        wait_seconds,
                        wait_for_html_frame,
                    } => {
                        let position = html_viewport_fraction_position(
                            &mut harness,
                            *x_fraction,
                            *y_fraction,
                        )?;
                        if *wait_for_html_frame {
                            click_at_and_wait_for_html_frame(
                                &mut harness,
                                recording.as_mut(),
                                position,
                                *button,
                                *wait_seconds,
                            )?;
                        } else {
                            click_at(&mut harness, position, *button);
                            step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                        }
                    }
                    UiAction::PressKey {
                        key,
                        wait_seconds,
                        wait_for_html_frame,
                    } => {
                        let key = screenshot_key(key)?;
                        if *wait_for_html_frame {
                            press_key_and_wait_for_html_frames(
                                &mut harness,
                                recording.as_mut(),
                                key,
                                *wait_seconds,
                            )?;
                        } else {
                            press_key(&mut harness, key);
                            step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                        }
                    }
                    UiAction::BurstHtmlInput {
                        count,
                        timeout_seconds,
                    } => {
                        dispatch_html_input_burst_and_wait(
                            &mut harness,
                            recording.as_mut(),
                            *count,
                            *timeout_seconds,
                        )?;
                    }
                    UiAction::ClickRgbRegion {
                        rgb,
                        tolerance,
                        min_region_pixels,
                        search_bounds,
                        button,
                        wait_seconds,
                    } => {
                        let image = harness.render().map_err(|error| {
                            anyhow::anyhow!("render failed before RGB click: {error}")
                        })?;
                        let effective_search_bounds = (*search_bounds).or_else(|| {
                            harness
                                .state_mut()
                                .html_browser_display_rect_for_test()
                                .and_then(|rect| {
                                    physical_png_bounds(
                                        rect,
                                        image.width(),
                                        image.height(),
                                        HARNESS_PIXELS_PER_POINT,
                                    )
                                })
                        });
                        let region = crate::capture::locate_largest_color_region_in_image(
                            &image,
                            *rgb,
                            *tolerance,
                            *min_region_pixels,
                            effective_search_bounds,
                        )
                        .map_err(|error| {
                            let diagnostic =
                                output_dir.join(format!("rgb-click-failure-step-{:02}.png", i + 1));
                            if let Err(save_error) = image.save(&diagnostic) {
                                return anyhow::anyhow!(
                                    "{error}; additionally failed to save {}: {save_error}",
                                    diagnostic.display()
                                );
                            }
                            anyhow::anyhow!(
                                "{error}; rendered frame saved to {}",
                                diagnostic.display()
                            )
                        })?;
                        let point = egui::pos2(
                            region.center_x as f32 / HARNESS_PIXELS_PER_POINT,
                            region.center_y as f32 / HARNESS_PIXELS_PER_POINT,
                        );
                        let browser_geometry = harness
                            .state_mut()
                            .html_browser_display_rect_for_test()
                            .zip(harness.state_mut().html_browser_frame_viewport_for_test())
                            .zip(
                                harness
                                    .state_mut()
                                    .html_browser_frame_scroll_metrics_for_test(),
                            );
                        println!(
                            "  click RGB({},{},{}): region {}..={}, {}..={} ({} pixels); harness point ({:.1}, {:.1}); browser geometry {browser_geometry:?}",
                            rgb[0],
                            rgb[1],
                            rgb[2],
                            region.min_x,
                            region.max_x,
                            region.min_y,
                            region.max_y,
                            region.pixels,
                            point.x,
                            point.y,
                        );
                        click_at(&mut harness, point, *button);
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    UiAction::TypeText { text, wait_seconds } => {
                        harness
                            .input_mut()
                            .events
                            .push(egui::Event::Text(text.clone()));
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    UiAction::ResizeWindow {
                        width,
                        height,
                        wait_seconds,
                    } => {
                        anyhow::ensure!(
                            *width > 0 && *height > 0,
                            "screenshot viewport dimensions must be positive"
                        );
                        harness.set_size(egui::vec2(*width as f32, *height as f32));
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    UiAction::DragByLabel {
                        from_label,
                        to_label,
                        move_steps,
                        hold_seconds,
                        wait_seconds,
                    } => {
                        perform_drag_by_labels(
                            &mut harness,
                            recording.as_mut(),
                            from_label,
                            to_label,
                            move_steps.as_ref().copied().unwrap_or(20),
                            hold_seconds.unwrap_or(0.0),
                            *wait_seconds,
                        )?;
                    }
                    other => {
                        let app_action = match other {
                            UiAction::ToggleToc => AppAction::ToggleToc,
                            UiAction::ToggleSplitView => {
                                AppAction::SetViewMode(katana_ui::app_state::ViewMode::Split)
                            }
                            UiAction::ToggleSettings => AppAction::ToggleSettings,
                            UiAction::ToggleExplorer => AppAction::ToggleExplorer,
                            UiAction::ToggleSlideshow => AppAction::ToggleSlideshow,
                            UiAction::ToggleStoryPanel => AppAction::ToggleStoryPanel,
                            UiAction::ToggleExportPanel => AppAction::ToggleExportPanel,
                            UiAction::OpenChangelog => AppAction::ShowReleaseNotes,
                            UiAction::OpenHelpDemo => AppAction::OpenHelpDemo,
                            UiAction::SelectNextTab => AppAction::SelectNextTab,
                            UiAction::RefreshDocument => {
                                AppAction::RefreshDocument { is_manual: true }
                            }
                            UiAction::ConfirmCurrentDiffReviewFile => {
                                AppAction::ConfirmCurrentDiffReviewFile
                            }
                            UiAction::OpenSettingsTab { .. }
                            | UiAction::OpenUrl { .. }
                            | UiAction::OpenFixtureUrl { .. }
                            | UiAction::OpenFixtureDocumentUrl { .. }
                            | UiAction::OpenFixtureDocumentErrorUrl { .. }
                            | UiAction::ForceOpenAccordion { .. }
                            | UiAction::OpenIconsAdvancedPanel
                            | UiAction::ScrollDown { .. }
                            | UiAction::SetScrollOffset { .. }
                            | UiAction::OpenFirstChangelogSection
                            | UiAction::SetViewMode { .. }
                            | UiAction::RunCommandPalette { .. }
                            | UiAction::RunGlobalSearch { .. }
                            | UiAction::RunDocumentSearch { .. }
                            | UiAction::SelectThemePresetInSettings { .. }
                            | UiAction::SlideshowNavigate { .. }
                            | UiAction::SelectDemoTab { .. }
                            | UiAction::OpenProblemsPanel
                            | UiAction::CloseSearchModal
                            | UiAction::CloseDocSearch
                            | UiAction::CloseActiveDocument { .. }
                            | UiAction::CloseAllDocuments { .. }
                            | UiAction::RunMixedDocumentCycles { .. }
                            | UiAction::RefreshDiagnostics
                            | UiAction::DocumentNext { .. }
                            | UiAction::ApplyLintFixesForActiveFile
                            | UiAction::ClickNode { .. }
                            | UiAction::HoverAt { .. }
                            | UiAction::ClickAt { .. }
                            | UiAction::ClickHtmlViewportFraction { .. }
                            | UiAction::PressKey { .. }
                            | UiAction::BurstHtmlInput { .. }
                            | UiAction::ClickRgbRegion { .. }
                            | UiAction::TypeText { .. }
                            | UiAction::ResizeWindow { .. }
                            | UiAction::DragByLabel { .. } => unreachable!(),
                        };
                        harness.state_mut().trigger_action(app_action);
                        let fps = recording.as_ref().map(|r| r.fps).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                }
            }
            Step::Drag(step) => {
                perform_drag_by_labels(
                    &mut harness,
                    recording.as_mut(),
                    &step.from_label,
                    &step.to_label,
                    step.move_steps(),
                    step.hold_seconds.unwrap_or(0.0),
                    step.wait_seconds,
                )?;
            }
            Step::Quit => {}
        }
        println!(
            "completed step {}/{} elapsed_ns={}",
            i + 1,
            steps.len(),
            operation_started.elapsed().as_nanos()
        );
    }
    if recording.is_some() {
        bail!("record_start was called but record_stop was not reached");
    }

    Ok(())
}

fn close_all_documents_and_wait_for_idle(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    timeout_seconds: f64,
) -> Result<()> {
    let deadline = async_assert_deadline(timeout_seconds)?;
    println!("  closing all documents: count={}", document_count(harness));
    close_extra_documents(harness, recording.as_deref_mut(), deadline)?;
    close_active_document_and_wait_for_idle(
        harness,
        recording,
        deadline
            .saturating_duration_since(Instant::now())
            .as_secs_f64(),
        "close_all_documents",
    )
}

fn close_extra_documents(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    deadline: Instant,
) -> Result<()> {
    while document_count(harness) > 1 {
        let active_index = harness
            .state_mut()
            .app_state_for_test()
            .document
            .active_doc_idx
            .context("close_all_documents requires an active document")?;
        harness
            .state_mut()
            .trigger_action(AppAction::ForceCloseDocument(active_index));
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        ensure!(
            Instant::now() < deadline,
            "close_all_documents exceeded its shared close budget"
        );
    }
    Ok(())
}

fn document_count(harness: &mut Harness<'_, KatanaApp>) -> usize {
    harness
        .state_mut()
        .app_state_for_test()
        .document
        .open_documents
        .len()
}

fn close_active_document_and_wait_for_idle(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    timeout_seconds: f64,
    operation: &str,
) -> Result<()> {
    let active_index = harness
        .state_mut()
        .app_state_for_test()
        .document
        .active_doc_idx
        .with_context(|| format!("{operation} requires an active document"))?;
    harness
        .state_mut()
        .trigger_action(AppAction::ForceCloseDocument(active_index));
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        let open_documents = harness
            .state_mut()
            .app_state_for_test()
            .document
            .open_documents
            .len();
        let resources = harness.state_mut().preview_resource_counts_for_test();
        let (document_workers, kdv_resources) =
            harness.state_mut().document_lifecycle_resources_for_test();
        let office_workers = current_office_worker_count()?;
        if open_documents == 0
            && resources == (0, 0, 0, 0, 0, 0)
            && office_workers == 0
            && document_workers == 0
            && kdv_resources == Default::default()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "{operation} did not release resources within {timeout_seconds:.2}s: open_documents={open_documents}, resources={resources:?}, office_workers={office_workers}, document_workers={document_workers}, kdv_resources={kdv_resources:?}"
            );
        }
        sleep_frame(60.0);
    }
}

fn capture_runtime_snapshot(
    harness: &mut Harness<'_, KatanaApp>,
    observed: ObservedFrameProgress,
) -> Result<RuntimeSnapshot> {
    let (previews, html_surfaces, document_surfaces, frames, textures, cache_entries) =
        harness.state_mut().preview_resource_counts_for_test();
    let (document_workers, kdv_resources) =
        harness.state_mut().document_lifecycle_resources_for_test();
    Ok(RuntimeSnapshot {
        rss_kib: current_process_rss_kib()?,
        ui_frame: harness.ctx.cumulative_frame_nr(),
        observed,
        previews,
        html_surfaces,
        document_surfaces,
        office_workers: current_office_worker_count()?,
        document_workers,
        kdv_resources,
        frames,
        textures,
        cache_entries,
    })
}

fn assert_runtime_snapshot(
    baseline: &RuntimeSnapshot,
    current: &RuntimeSnapshot,
    expected: &crate::request::AssertRuntimeSnapshotStep,
) -> Result<()> {
    let rss_delta_kib = current.rss_kib.saturating_sub(baseline.rss_kib);
    let ui_frame_delta = current.ui_frame.saturating_sub(baseline.ui_frame);
    let html_frame_delta = current.observed.html.saturating_sub(baseline.observed.html);
    let document_frame_delta = current
        .observed
        .document
        .saturating_sub(baseline.observed.document);
    ensure!(
        rss_delta_kib <= expected.max_rss_delta_kib,
        "RSS delta exceeded runtime budget: {rss_delta_kib} KiB > {} KiB; baseline={baseline:?}, current={current:?}",
        expected.max_rss_delta_kib
    );
    ensure!(
        ui_frame_delta >= expected.min_ui_frame_delta,
        "UI heartbeat did not make enough progress: {ui_frame_delta} < {}; baseline={baseline:?}, current={current:?}",
        expected.min_ui_frame_delta
    );
    ensure!(
        html_frame_delta >= expected.min_html_frames_observed,
        "HTML generation progress is incomplete: {html_frame_delta} < {}; baseline={baseline:?}, current={current:?}",
        expected.min_html_frames_observed
    );
    ensure!(
        document_frame_delta >= expected.min_document_frames_observed,
        "document generation progress is incomplete: {document_frame_delta} < {}; baseline={baseline:?}, current={current:?}",
        expected.min_document_frames_observed
    );
    if expected.max_document_surfaces == 0 {
        ensure!(
            current.document_workers == 0 && current.kdv_resources == Default::default(),
            "document lifecycle resources remain after preview removal: current={current:?}"
        );
    }
    ensure!(
        current.previews <= expected.max_previews
            && current.html_surfaces <= expected.max_html_surfaces
            && current.document_surfaces <= expected.max_document_surfaces
            && current.office_workers <= expected.max_office_workers
            && current.frames <= expected.max_frames
            && current.textures <= expected.max_textures
            && current.cache_entries <= expected.max_cache_entries,
        "preview resources did not return to the configured bound: current={current:?}, limits=(previews={}, html_surfaces={}, document_surfaces={}, office_workers={}, frames={}, textures={}, cache_entries={})",
        expected.max_previews,
        expected.max_html_surfaces,
        expected.max_document_surfaces,
        expected.max_office_workers,
        expected.max_frames,
        expected.max_textures,
        expected.max_cache_entries
    );
    Ok(())
}

#[cfg(unix)]
fn current_process_rss_kib() -> Result<u64> {
    let process_id = std::process::id().to_string();
    let mut command = ProcessService::create_command("ps");
    command.args(["-o", "rss=", "-p", process_id.as_str()]);
    let output =
        ProcessService::output(command).context("failed to sample screenshot-runner RSS")?;
    ensure!(output.status.success(), "ps failed while sampling RSS");
    String::from_utf8(output.stdout)
        .context("ps RSS output was not UTF-8")?
        .trim()
        .parse::<u64>()
        .context("ps RSS output was not an integer KiB value")
}

#[cfg(not(unix))]
fn current_process_rss_kib() -> Result<u64> {
    bail!("runtime RSS snapshots are currently supported on Unix targets")
}

#[cfg(unix)]
fn current_office_worker_count() -> Result<usize> {
    let process_id = std::process::id();
    let mut command = ProcessService::create_command("ps");
    command.args(["-axo", "ppid=,comm="]);
    let output =
        ProcessService::output(command).context("failed to enumerate Office worker processes")?;
    ensure!(
        output.status.success(),
        "ps failed while enumerating Office workers"
    );
    let stdout = String::from_utf8(output.stdout).context("ps worker output was not UTF-8")?;
    Ok(stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let parent = fields.next()?.parse::<u32>().ok()?;
            let command = fields.next()?;
            Some(parent == process_id && command.contains("kdv-office-worker"))
        })
        .filter(|matches| *matches)
        .count())
}

#[cfg(not(unix))]
fn current_office_worker_count() -> Result<usize> {
    bail!("Office worker process snapshots are currently supported on Unix targets")
}

fn physical_png_bounds(
    rect: egui::Rect,
    image_width: u32,
    image_height: u32,
    pixels_per_point: f32,
) -> Option<PngBounds> {
    if !rect.is_finite() || !pixels_per_point.is_finite() || pixels_per_point <= 0.0 {
        return None;
    }
    let min_x = (rect.min.x * pixels_per_point)
        .floor()
        .clamp(0.0, image_width as f32) as u32;
    let min_y = (rect.min.y * pixels_per_point)
        .floor()
        .clamp(0.0, image_height as f32) as u32;
    let max_x = (rect.max.x * pixels_per_point)
        .ceil()
        .clamp(0.0, image_width as f32) as u32;
    let max_y = (rect.max.y * pixels_per_point)
        .ceil()
        .clamp(0.0, image_height as f32) as u32;
    (max_x > min_x && max_y > min_y).then_some(PngBounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

fn scroll_delta(direction: ScrollDirection, amount: f32) -> egui::Vec2 {
    match direction {
        ScrollDirection::Up => egui::vec2(0.0, amount),
        ScrollDirection::Down => egui::vec2(0.0, -amount),
        ScrollDirection::Left => egui::vec2(amount, 0.0),
        ScrollDirection::Right => egui::vec2(-amount, 0.0),
    }
}

fn assert_active_document(
    harness: &mut Harness<'_, KatanaApp>,
    assertion: &AssertActiveDocumentStep,
) -> Result<()> {
    harness.run_steps(120);
    let path = {
        let app_state = harness.state_mut().app_state_mut();
        let document = app_state
            .active_document()
            .context("expected an active document, but no document is active")?;
        document.path.to_string_lossy().to_string()
    };
    if !path.contains(&assertion.path_contains) {
        bail!(
            "active document assertion failed: expected path to contain {:?}, got {:?}",
            assertion.path_contains,
            path
        );
    }
    println!("  active document matched: {path}");
    Ok(())
}

fn assert_html_browser_origin(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    assertion: &AssertHtmlBrowserOriginStep,
) -> Result<()> {
    let deadline = async_assert_deadline(assertion.timeout_seconds)?;
    loop {
        let origin = harness.state_mut().html_browser_origin_for_test();
        if origin
            .as_deref()
            .is_some_and(|origin| origin.ends_with(&assertion.origin_ends_with))
        {
            println!(
                "  HTML browser origin matched: {}",
                origin.unwrap_or_default()
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "HTML browser origin did not end with {:?} within {:.2}s; last origin was {:?}",
                assertion.origin_ends_with,
                assertion.timeout_seconds,
                origin
            );
        }
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        sleep_frame(60.0);
    }
}

fn assert_html_browser_frame_contains_rgb(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    assertion: &AssertHtmlBrowserFrameContainsRgbStep,
) -> Result<()> {
    let deadline = async_assert_deadline(assertion.timeout_seconds)?;
    loop {
        let matching = harness
            .state_mut()
            .html_browser_frame_matching_rgb_pixels_for_test(assertion.rgb)
            .unwrap_or(0);
        if matching >= assertion.min_pixels {
            println!("  matching KRR frame RGB pixels: {matching}");
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "KRR frame contains {matching} pixels at rgb({},{},{}), expected at least {} within {:.2}s",
                assertion.rgb[0],
                assertion.rgb[1],
                assertion.rgb[2],
                assertion.min_pixels,
                assertion.timeout_seconds
            );
        }
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        sleep_frame(60.0);
    }
}

fn assert_http_requests(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    server: &FixtureHttpServer,
    assertion: &crate::request::AssertHttpRequestsStep,
) -> Result<()> {
    let deadline = async_assert_deadline(assertion.timeout_seconds)?;
    loop {
        let actual = server.requested_paths()?;
        let missing = assertion
            .paths
            .iter()
            .filter(|path| !actual.contains(path))
            .cloned()
            .collect::<Vec<_>>();
        if missing.is_empty() {
            server.assert_requested(&assertion.paths)?;
            return Ok(());
        }
        if Instant::now() >= deadline {
            let app = harness.state_mut();
            let pending = format!("{:?}", app.pending_action_for_test());
            let url_tab = &app.app_state_for_test().url_tab;
            let url_loading = url_tab.is_loading;
            let url_input = &url_tab.input;
            let url_error = &url_tab.last_error;
            let active_path = app.app_state_for_test().active_path();
            bail!(
                "fixture HTTP requests are missing {missing:?} after {:.2}s; received {actual:?}; pending_action={pending}; url_loading={url_loading}; url_input={url_input:?}; url_error={url_error:?}; active_path={active_path:?}",
                assertion.timeout_seconds,
            );
        }
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        sleep_frame(60.0);
    }
}

fn async_assert_deadline(timeout_seconds: f64) -> Result<Instant> {
    ensure!(
        timeout_seconds.is_finite() && timeout_seconds > 0.0,
        "async assertion timeout must be a positive finite number"
    );
    Ok(Instant::now() + Duration::from_secs_f64(timeout_seconds))
}

fn wait_for_browser_scroll(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    initial_scroll: f32,
    direction: ScrollDirection,
    pixels: f32,
) -> Result<()> {
    const SCROLL_TIMEOUT_SECONDS: f64 = 10.0;
    const SCROLL_TOLERANCE: f32 = 1.0;
    let deadline = async_assert_deadline(SCROLL_TIMEOUT_SECONDS)?;
    loop {
        let (scroll_y, content_height) = harness
            .state_mut()
            .html_browser_frame_scroll_metrics_for_test()
            .context("expected HTML browser scroll metrics")?;
        let viewport_height = harness
            .state_mut()
            .html_browser_frame_viewport_for_test()
            .context("expected HTML browser viewport while waiting for scroll")?
            .1;
        let max_scroll = (content_height - viewport_height).max(0.0);
        let expected = match direction {
            ScrollDirection::Down => (initial_scroll + pixels).min(max_scroll),
            ScrollDirection::Up => (initial_scroll - pixels).max(0.0),
            ScrollDirection::Left | ScrollDirection::Right => initial_scroll,
        };
        if (scroll_y - expected).abs() <= SCROLL_TOLERANCE {
            println!("  HTML browser scroll settled: {scroll_y:.2}/{max_scroll:.2}");
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "HTML browser scroll did not settle within {SCROLL_TIMEOUT_SECONDS:.2}s: expected {expected:.2}, observed {scroll_y:.2}, max {max_scroll:.2}"
            );
        }
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        sleep_frame(60.0);
    }
}

fn assert_html_browser_viewport_matches_display_rect(
    harness: &mut Harness<'_, KatanaApp>,
) -> Result<()> {
    harness.run_steps(120);
    let app = harness.state_mut();
    let display = app
        .html_browser_display_rect_for_test()
        .context("expected an active HTML display rect")?;
    let viewport = app
        .html_browser_frame_viewport_for_test()
        .context("expected an active KRR frame viewport")?;
    let display_size = (display.width(), display.height());
    ensure!(
        (viewport.0 - display_size.0).abs() <= 1.0 && (viewport.1 - display_size.1).abs() <= 1.0,
        "KRR viewport {viewport:?} does not match HTML display rect {display_size:?}"
    );
    println!("  KRR viewport matches HTML display rect: {viewport:?}");
    Ok(())
}

fn assert_diff_review(
    harness: &mut Harness<'_, KatanaApp>,
    assertion: &AssertDiffReviewStep,
) -> Result<()> {
    harness.run_steps(120);
    let snapshot = harness
        .state_mut()
        .app_state_mut()
        .layout
        .diff_review_snapshot()
        .context("expected an active diff review, but no diff review is active")?;

    if let Some(expected_count) = assertion.file_count {
        if snapshot.file_count != expected_count {
            bail!(
                "diff review assertion failed: expected {expected_count} files, got {}",
                snapshot.file_count
            );
        }
    }

    if let Some(expected) = &assertion.target_path_contains {
        if !snapshot.target_path.contains(expected) {
            bail!(
                "diff review assertion failed: expected target path to contain {:?}, got {:?}",
                expected,
                snapshot.target_path
            );
        }
    }
    if let Some(expected) = &assertion.before_contains {
        if !snapshot.before.contains(expected) {
            bail!(
                "diff review assertion failed: expected before content to contain {:?}",
                expected
            );
        }
    }
    if let Some(expected) = &assertion.after_contains {
        if !snapshot.after.contains(expected) {
            bail!(
                "diff review assertion failed: expected after content to contain {:?}",
                expected
            );
        }
    }

    println!("  diff review matched: {}", snapshot.target_path);
    Ok(())
}

fn maybe_capture_recording_frame(
    harness: &mut Harness<'_, KatanaApp>,
    recording: Option<&mut ActiveRecording>,
) -> Result<()> {
    if let Some(recorder) = recording {
        let fps = recorder.fps as f64;
        let frame_step_seconds = 1.0 / fps;
        if recorder.should_capture_this_tick(frame_step_seconds) {
            recorder.capture_frame(harness)?;
        }
    }
    Ok(())
}

fn encode_video(recorder: &ActiveRecording, output_path: &Path) -> Result<()> {
    let input_pattern = recorder.frame_dir.path().join("frame_%06d.png");
    let mut cmd = ProcessService::create_command("ffmpeg");
    cmd.arg("-y")
        .arg("-framerate")
        .arg(recorder.fps.to_string())
        .arg("-i")
        .arg(&input_pattern)
        .arg("-an");

    match recorder.format {
        VideoFormat::Webm => {
            cmd.arg("-c:v")
                .arg("libvpx-vp9")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-b:v")
                .arg("0")
                .arg("-crf")
                .arg("32")
                .arg("-row-mt")
                .arg("1")
                .arg("-cpu-used")
                .arg("4");
        }
        VideoFormat::Mp4 => {
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-preset")
                .arg("veryfast")
                .arg("-crf")
                .arg("23")
                .arg("-movflags")
                .arg("+faststart");
        }
    }

    cmd.arg(output_path)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let status = cmd
        .status()
        .context("failed to start ffmpeg (install ffmpeg for video recording steps)")?;
    if !status.success() {
        bail!("ffmpeg failed to encode video: {}", output_path.display());
    }
    Ok(())
}

fn run_command_palette(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    query: &str,
    katana_mode: bool,
    execute_first: bool,
    keystroke_delay_seconds: f64,
    pause_after_seconds: f64,
) -> Result<()> {
    let open_action = if katana_mode {
        AppAction::ToggleKatanaCommandPalette
    } else {
        AppAction::ToggleCommandPalette
    };
    harness.state_mut().trigger_action(open_action);
    step_for_seconds(harness, recording.as_deref_mut(), 0.25)?;

    let prefix = if katana_mode { ">" } else { "" };
    let mut typed = String::from(prefix);
    for ch in query.chars() {
        typed.push(ch);
        {
            let app = harness.state_mut().app_state_mut();
            app.command_palette.current_query = typed.clone();
        }
        refresh_command_palette_results(harness);
        step_for_seconds(harness, recording.as_deref_mut(), keystroke_delay_seconds)?;
    }

    step_for_seconds(harness, recording.as_deref_mut(), pause_after_seconds)?;

    if execute_first {
        let first = harness
            .state_mut()
            .app_state_mut()
            .command_palette
            .results
            .first()
            .cloned()
            .context("command palette had no matching result")?;
        execute_palette_result(harness, &first);
        {
            let app = harness.state_mut().app_state_mut();
            app.command_palette.is_open = false;
        }
        step_for_seconds(harness, recording, 0.45)?;
    }

    Ok(())
}

fn refresh_command_palette_results(harness: &mut Harness<'_, KatanaApp>) {
    let providers: Vec<Box<dyn CommandPaletteProvider>> = vec![
        Box::new(AppCommandProvider),
        Box::new(WorkspaceFileProvider),
        Box::new(MarkdownContentProvider),
    ];
    let app = harness.state_mut().app_state_mut();
    let is_action_mode = app.command_palette.current_query.starts_with('>');
    let actual_query = if is_action_mode {
        app.command_palette.current_query[1..]
            .trim_start()
            .to_string()
    } else {
        app.command_palette.current_query.clone()
    };
    let workspace = app.workspace.data.as_ref();
    let mut gathered = Vec::new();
    for provider in providers {
        if is_action_mode && provider.name() != "Commands" {
            continue;
        }
        if !is_action_mode && provider.name() == "Commands" {
            continue;
        }
        gathered.extend(provider.search(&actual_query, workspace, None));
    }
    gathered.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    app.command_palette.update_results(gathered);
}

fn execute_palette_result(harness: &mut Harness<'_, KatanaApp>, result: &CommandPaletteResult) {
    let app_action = match &result.execute_payload {
        CommandPaletteExecutePayload::DispatchAppAction(action) => action.clone(),
        CommandPaletteExecutePayload::OpenFile(path) => AppAction::SelectDocument(path.clone()),
        CommandPaletteExecutePayload::NavigateToContent {
            path,
            line,
            byte_range,
        } => AppAction::SelectDocumentAndJump {
            path: path.clone(),
            line: *line,
            byte_range: byte_range.clone(),
        },
    };
    harness.state_mut().trigger_action(app_action);
}

fn run_global_search(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    query: &str,
    tab: &str,
    keystroke_delay_seconds: f64,
    pause_after_seconds: f64,
) -> Result<()> {
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSearchModal);
    step_for_seconds(harness, recording.as_deref_mut(), 0.3)?;
    {
        let search = &mut harness.state_mut().app_state_mut().search;
        search.active_tab = if tab == "markdown_content" {
            katana_ui::app_state::SearchTab::MarkdownContent
        } else {
            katana_ui::app_state::SearchTab::FileName
        };
        search.focus_requested = false;
        search.file_search.query.clear();
        search.md_search.query.clear();
    }

    let mut typed = String::new();
    for ch in query.chars() {
        typed.push(ch);
        apply_global_search_query(harness, tab, &typed);
        step_for_seconds(harness, recording.as_deref_mut(), keystroke_delay_seconds)?;
    }
    step_for_seconds(harness, recording, pause_after_seconds)?;
    Ok(())
}

fn apply_global_search_query(harness: &mut Harness<'_, KatanaApp>, tab: &str, query: &str) {
    let app = harness.state_mut().app_state_mut();
    if tab == "markdown_content" {
        app.search.md_search.query = query.to_string();
        let workspace_root = app.workspace.data.as_ref().map(|ws| ws.root.clone());
        app.search.md_last_params = Some((app.search.md_search.clone(), workspace_root));
        if let Some(ws) = app.workspace.data.as_ref() {
            app.search.md_results = katana_core::search::WorkspaceSearchOps::search_workspace(
                ws,
                query,
                app.search.md_search.match_case,
                app.search.md_search.match_word,
                app.search.md_search.use_regex,
                50,
            );
        }
    } else {
        app.search.file_search.query = query.to_string();
        let mut matches = Vec::new();
        if let Some(ws) = app.workspace.data.as_ref() {
            katana_ui::shell_logic::ShellLogicOps::collect_matches(
                &ws.tree,
                &query.to_lowercase(),
                &[],
                &[],
                &ws.root,
                false,
                false,
                false,
                &mut matches,
            );
        }
        app.search.results = matches;
    }
}

fn run_document_search(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    query: &str,
    next_count: u32,
    keystroke_delay_seconds: f64,
    pause_after_seconds: f64,
) -> Result<()> {
    harness.state_mut().trigger_action(AppAction::OpenDocSearch);
    step_for_seconds(harness, recording.as_deref_mut(), 0.25)?;

    let mut typed = String::new();
    for ch in query.chars() {
        typed.push(ch);
        {
            let search = &mut harness.state_mut().app_state_mut().search;
            search.doc_search.query = typed.clone();
        }
        harness
            .state_mut()
            .trigger_action(AppAction::DocSearchQueryChanged);
        step_for_seconds(harness, recording.as_deref_mut(), keystroke_delay_seconds)?;
    }

    for _ in 0..next_count {
        harness.state_mut().trigger_action(AppAction::DocSearchNext);
        step_for_seconds(harness, recording.as_deref_mut(), 0.25)?;
    }

    step_for_seconds(harness, recording, pause_after_seconds)?;
    Ok(())
}

fn select_theme_preset_in_settings(
    harness: &mut Harness<'_, KatanaApp>,
    recording: Option<&mut ActiveRecording>,
    preset: &str,
) -> Result<()> {
    {
        let app = harness.state_mut().app_state_mut();
        if !app.layout.show_settings {
            bail!("theme preset selection requires settings window to be open");
        }
        if app.config.active_settings_tab != SettingsTab::Theme {
            bail!("theme preset selection requires Settings > Theme to be active");
        }
    }
    let theme_preset = match preset {
        "katana_dark" => ThemePreset::KatanaDark,
        "katana_light" => ThemePreset::KatanaLight,
        other => bail!("unsupported theme preset for demo: {other}"),
    };
    {
        let app = harness.state_mut().app_state_mut();
        let settings = app.config.settings.settings_mut();
        settings.theme.preset = theme_preset;
        settings.theme.theme = match theme_preset.colors().mode {
            ThemeMode::Dark => "dark".to_string(),
            ThemeMode::Light => "light".to_string(),
        };
        settings.theme.active_custom_theme = None;
        settings.theme.custom_color_overrides = None;
        let _ = app.config.try_save_settings();
    }
    step_for_seconds(harness, recording, 1.2)
}

fn navigate_slideshow(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    direction: &str,
    steps: u32,
    wait_seconds: f64,
) -> Result<()> {
    let delta: i32 = match direction {
        "next" | "right" => 1,
        "prev" | "left" => -1,
        other => bail!("unsupported slideshow direction: {other}"),
    };
    for _ in 0..steps {
        let layout = &mut harness.state_mut().app_state_mut().layout;
        if delta > 0 {
            layout.slideshow_page += 1;
        } else {
            layout.slideshow_page = layout.slideshow_page.saturating_sub(1);
        }
        step_for_seconds(harness, recording.as_deref_mut(), wait_seconds)?;
    }
    Ok(())
}

fn assert_node_bounds(rect: egui::Rect, viewport: egui::Rect, bounds: [f32; 4]) -> Result<()> {
    ensure!(
        bounds.iter().all(|value| value.is_finite()),
        "node bounds must be finite"
    );
    let region = egui::Rect::from_min_max(
        egui::pos2(bounds[0], bounds[1]),
        egui::pos2(bounds[2], bounds[3]),
    );
    ensure!(
        region.is_positive() && viewport.contains_rect(region),
        "expected node region must be positive and inside viewport: {region:?}"
    );
    ensure!(
        rect.is_finite() && rect.is_positive() && region.contains_rect(rect),
        "node escaped expected region: rect={rect:?}, region={region:?}"
    );
    println!("  node bounds verified: {rect:?} inside {region:?}");
    Ok(())
}

fn click_node<State>(harness: &mut Harness<'_, State>, label: &str, button: ClickButton) {
    let viewport = harness.ctx.viewport_rect();
    let all_rects: Vec<_> = harness
        .get_all_by_label(label)
        .map(|node| node.rect())
        .collect();
    let visible_rects: Vec<_> = all_rects
        .iter()
        .copied()
        .filter(|rect| viewport.intersects(*rect))
        .collect();
    let [logical_rect] = visible_rects.as_slice() else {
        panic!(
            "expected exactly one visible accessibility node {label:?}, found {}: {visible_rects:?}; all: {all_rects:?}",
            visible_rects.len()
        );
    };
    // Node::rect uses logical coordinates, so reapplying the scale would shift the click target.
    let logical_pos = logical_rect.center();
    println!("  click accessibility node {label:?}: {logical_rect:?} -> {logical_pos:?}");
    click_at(harness, logical_pos, button);
}

fn apply_lint_fixes_for_active_file(
    harness: &mut Harness<'_, KatanaApp>,
    recording: Option<&mut ActiveRecording>,
) -> Result<()> {
    let batch = {
        let app = harness.state_mut().app_state_mut();
        let active_path = app
            .active_document()
            .context("expected an active document before applying lint fixes")?
            .path
            .clone();
        let fixes: Vec<_> = app
            .diagnostics
            .get_file_diagnostics(&active_path)
            .iter()
            .filter(|diagnostic| diagnostic.official_meta.is_some())
            .filter_map(|diagnostic| diagnostic.fix_info.clone())
            .collect();
        if fixes.is_empty() {
            bail!("active document has no applicable lint fixes: {active_path:?}");
        }
        katana_ui::app_action::LintFixBatch {
            source: app
                .diagnostics
                .content_snapshot(&active_path)
                .map(str::to_string),
            path: active_path,
            fixes,
        }
    };
    harness
        .state_mut()
        .trigger_action(AppAction::ApplyLintFixesForFiles(vec![batch]));
    step_for_seconds(harness, recording, 1.0)?;
    Ok(())
}

fn click_at<State>(harness: &mut Harness<'_, State>, pos: egui::Pos2, button: ClickButton) {
    move_pointer(harness, pos);
    send_pointer_button_state(harness, pos, button, true);
    send_pointer_button_state(harness, pos, button, false);
}

fn move_pointer<State>(harness: &mut Harness<'_, State>, pos: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(pos));
    harness.step();
}

fn send_pointer_button_state<State>(
    harness: &mut Harness<'_, State>,
    pos: egui::Pos2,
    button: ClickButton,
    pressed: bool,
) {
    let pointer_button = match button {
        ClickButton::Primary => egui::PointerButton::Primary,
        ClickButton::Secondary => egui::PointerButton::Secondary,
    };
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos,
        button: pointer_button,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
}

fn screenshot_key(key: &str) -> Result<egui::Key> {
    match key.trim().to_ascii_lowercase().as_str() {
        "arrowdown" | "down" => Ok(egui::Key::ArrowDown),
        "arrowleft" | "left" => Ok(egui::Key::ArrowLeft),
        "arrowright" | "right" => Ok(egui::Key::ArrowRight),
        "arrowup" | "up" => Ok(egui::Key::ArrowUp),
        "end" => Ok(egui::Key::End),
        "enter" => Ok(egui::Key::Enter),
        "escape" | "esc" => Ok(egui::Key::Escape),
        "home" => Ok(egui::Key::Home),
        "space" => Ok(egui::Key::Space),
        value => anyhow::bail!("unsupported screenshot key {value:?}"),
    }
}

fn press_key(harness: &mut Harness<'_, KatanaApp>, key: egui::Key) {
    for pressed in [true, false] {
        send_key_state(harness, key, pressed);
    }
}

fn send_key_state(harness: &mut Harness<'_, KatanaApp>, key: egui::Key, pressed: bool) {
    harness.input_mut().events.push(egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
}

fn press_key_and_wait_for_html_frames(
    harness: &mut Harness<'_, KatanaApp>,
    recording: Option<&mut ActiveRecording>,
    key: egui::Key,
    timeout_seconds: f64,
) -> Result<()> {
    let previous_generation = harness
        .state_mut()
        .html_browser_frame_generation_for_test()
        .context("wait_for_html_frame requires an active HTML browser frame")?;
    send_key_state(harness, key, true);
    send_key_state(harness, key, false);
    wait_for_html_browser_frame_advance_and_idle(
        harness,
        recording,
        previous_generation,
        timeout_seconds,
    )
}

fn dispatch_html_input_burst_and_wait(
    harness: &mut Harness<'_, KatanaApp>,
    recording: Option<&mut ActiveRecording>,
    count: u32,
    timeout_seconds: f64,
) -> Result<()> {
    ensure!(count > 0, "HTML browser input burst count must be positive");
    let previous_generation = current_html_browser_frame_generation(harness)?;
    harness
        .state_mut()
        .dispatch_html_browser_input_burst_for_test(count)
        .map_err(anyhow::Error::msg)?;
    wait_for_html_browser_frame_advance_and_idle(
        harness,
        recording,
        previous_generation,
        timeout_seconds,
    )
}

fn perform_drag_by_labels(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    from_label: &str,
    to_label: &str,
    move_steps: u32,
    hold_seconds: f64,
    wait_seconds: f64,
) -> Result<()> {
    let move_steps = move_steps.max(1);

    let from_node = harness.get_by_label(from_label);
    let to_node = harness.get_by_label(to_label);
    let from_pos = from_node.rect().center();
    let to_pos = to_node.rect().center();

    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(from_pos));
    harness.step();
    maybe_capture_recording_frame(harness, recording.as_deref_mut())?;

    if hold_seconds > 0.0 {
        step_for_seconds(harness, recording.as_deref_mut(), hold_seconds)?;
    }

    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: from_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    maybe_capture_recording_frame(harness, recording.as_deref_mut())?;

    for i in 1..=move_steps {
        let t = (i as f32) / (move_steps as f32);
        let current = egui::pos2(
            from_pos.x + (to_pos.x - from_pos.x) * t,
            from_pos.y + (to_pos.y - from_pos.y) * t,
        );
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(current));
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        sleep_frame(60.0);
    }

    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: to_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    maybe_capture_recording_frame(harness, recording.as_deref_mut())?;

    step_for_seconds(harness, recording, wait_seconds)?;
    Ok(())
}

fn step_for_seconds(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    seconds: f64,
) -> Result<()> {
    let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
    let frames = ((seconds * fps) as usize).max(1);
    for _ in 0..frames {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        sleep_frame(fps);
    }
    Ok(())
}

fn step_opens_document(step: &Step) -> bool {
    match step {
        Step::OpenFile(_) => true,
        Step::Action(action) => matches!(
            &action.action,
            UiAction::OpenUrl { .. }
                | UiAction::OpenFixtureUrl { .. }
                | UiAction::OpenFixtureDocumentUrl { .. }
                | UiAction::OpenFixtureDocumentErrorUrl { .. }
        ),
        _ => false,
    }
}

fn html_browser_frame_identity(
    harness: &mut Harness<'_, KatanaApp>,
) -> Option<HtmlBrowserFrameIdentity> {
    let app = harness.state_mut();
    Some(HtmlBrowserFrameIdentity {
        document_path: app.app_state_for_test().active_path()?,
        origin: app.html_browser_origin_for_test()?,
        generation: app.html_browser_frame_generation_for_test()?,
    })
}

fn document_frame_identity(harness: &mut Harness<'_, KatanaApp>) -> Option<DocumentFrameIdentity> {
    let (document_path, format, active_index, item_count, node_kind) =
        harness.state_mut().document_frame_for_test()?;
    Some(DocumentFrameIdentity {
        document_path,
        format,
        active_index,
        item_count,
        node_kind,
    })
}

fn wait_for_document_frame(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    previous_frame: Option<DocumentFrameIdentity>,
    previous_failure: Option<String>,
    timeout_seconds: f64,
) -> Result<(Duration, DocumentFrameIdentity)> {
    let started_at = Instant::now();
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        let failure = harness.state_mut().document_failure_for_test();
        if document_failure_advanced(previous_failure.as_deref(), failure.as_deref()) {
            let failure = failure.expect("advanced document failure must exist");
            bail!("document viewer failed before its first frame:\n{failure}");
        }
        let current = document_frame_identity(harness);
        let idle = harness.state_mut().document_is_idle_for_test() == Some(true);
        if idle && current.as_ref() != previous_frame.as_ref() {
            if let Some(current) = current {
                let elapsed = started_at.elapsed();
                println!(
                    "  document first frame ready in {:.3}s: format={}, item={}/{}, node={}",
                    elapsed.as_secs_f64(),
                    current.format,
                    current.active_index.saturating_add(1),
                    current.item_count,
                    current.node_kind,
                );
                return Ok((elapsed, current));
            }
        }
        if Instant::now() >= deadline {
            bail!("document viewer did not produce an initial frame within {timeout_seconds:.2}s");
        }
        sleep_frame(60.0);
    }
}

fn wait_for_document_failure(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    previous: Option<&str>,
    expected: &str,
    timeout_seconds: f64,
) -> Result<()> {
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        if let Some(failure) = harness.state_mut().document_failure_for_test() {
            if previous != Some(failure.as_str()) {
                ensure!(
                    failure.contains(expected),
                    "document failure did not contain {expected:?}: {failure}"
                );
                println!("  document produced expected failure: {failure}");
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            bail!("document did not fail within {timeout_seconds:.2}s");
        }
        sleep_frame(60.0);
    }
}

fn assert_document_frame(
    expected_format: Option<&str>,
    expected_item_count: Option<usize>,
    expected_node_kind: Option<&str>,
    frame: &DocumentFrameIdentity,
) -> Result<()> {
    if let Some(expected) = expected_format {
        ensure!(
            frame.format == expected,
            "document format mismatch: expected {expected:?}, got {:?}",
            frame.format
        );
    }
    if let Some(expected) = expected_item_count {
        ensure!(
            frame.item_count == expected,
            "document item count mismatch: expected {expected}, got {}",
            frame.item_count
        );
    }
    if let Some(expected) = expected_node_kind {
        ensure!(
            frame.node_kind == expected,
            "document node kind mismatch: expected {expected:?}, got {:?}",
            frame.node_kind
        );
    }
    Ok(())
}

fn wait_for_document_surface_idle(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    timeout_seconds: f64,
    operation: &str,
) -> Result<()> {
    if document_frame_identity(harness).is_none() {
        return Ok(());
    }
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        if harness.state_mut().document_failure_for_test().is_some()
            || harness.state_mut().document_is_idle_for_test() == Some(true)
        {
            return Ok(());
        }
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        if Instant::now() >= deadline {
            let frame = document_frame_identity(harness);
            let idle = harness.state_mut().document_is_idle_for_test();
            let failure = harness.state_mut().document_failure_for_test();
            bail!(
                "document surface did not settle before {operation} within \
                 {timeout_seconds:.2}s: frame={frame:?}, idle={idle:?}, failure={failure:?}"
            );
        }
        sleep_frame(60.0);
    }
}

fn advance_document_and_wait(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    timeout_seconds: f64,
) -> Result<()> {
    wait_for_document_surface_idle(
        harness,
        recording.as_deref_mut(),
        DOCUMENT_SCREENSHOT_SETTLE_TIMEOUT_SECONDS,
        "document navigation",
    )?;
    let previous = document_frame_identity(harness)
        .context("document_next requires an active document frame")?;
    harness
        .state_mut()
        .document_next_for_test()
        .map_err(anyhow::Error::msg)?;
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        if let Some(failure) = harness.state_mut().document_failure_for_test() {
            bail!("document viewer failed while advancing:\n{failure}");
        }
        if let Some(current) = document_frame_identity(harness) {
            let idle = harness.state_mut().document_is_idle_for_test() == Some(true);
            if idle
                && current.document_path == previous.document_path
                && current.active_index == previous.active_index.saturating_add(1)
            {
                println!(
                    "  document advanced: format={}, item={}/{}",
                    current.format,
                    current.active_index.saturating_add(1),
                    current.item_count
                );
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            let current = document_frame_identity(harness);
            let idle = harness.state_mut().document_is_idle_for_test();
            let failure = harness.state_mut().document_failure_for_test();
            bail!(
                "document viewer did not advance from item {} within {timeout_seconds:.2}s: \
                 current={current:?}, idle={idle:?}, failure={failure:?}",
                previous.active_index.saturating_add(1),
            );
        }
        sleep_frame(60.0);
    }
}

fn html_browser_frame_advanced(
    previous: Option<&HtmlBrowserFrameIdentity>,
    current: Option<&HtmlBrowserFrameIdentity>,
) -> bool {
    current.is_some_and(|current| Some(current) != previous)
}

fn document_failure_advanced(previous: Option<&str>, current: Option<&str>) -> bool {
    current.is_some_and(|current| Some(current) != previous)
}

fn wait_for_html_browser_frame_advance_and_idle(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    previous_generation: u64,
    timeout_seconds: f64,
) -> Result<()> {
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        let generation = harness.state_mut().html_browser_frame_generation_for_test();
        let idle = harness.state_mut().html_browser_is_idle_for_test();
        if generation.is_some_and(|current| current > previous_generation) && idle == Some(true) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "HTML browser did not drain after generation {previous_generation} within {timeout_seconds:.2}s (latest generation: {generation:?}, adapter idle: {idle:?})"
            );
        }
        sleep_frame(60.0);
    }
}

fn wait_for_html_browser_idle(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    timeout_seconds: f64,
) -> Result<()> {
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        let idle = harness.state_mut().html_browser_is_idle_for_test();
        if idle == Some(true) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "HTML browser did not become idle within {timeout_seconds:.2}s (adapter idle: {idle:?})"
            );
        }
        sleep_frame(60.0);
    }
}

fn click_at_and_wait_for_html_frame(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    position: egui::Pos2,
    button: ClickButton,
    timeout_seconds: f64,
) -> Result<()> {
    let display_rect = harness
        .state_mut()
        .html_browser_display_rect_for_test()
        .context("wait_for_html_frame requires an active HTML display rect")?;
    let staging_position = click_staging_position(display_rect, position);

    move_pointer(harness, staging_position);
    move_pointer(harness, position);
    wait_for_html_browser_idle(harness, recording.as_deref_mut(), timeout_seconds)?;
    let hover_generation = current_html_browser_frame_generation(harness)?;
    send_pointer_button_state(harness, position, button, true);
    send_pointer_button_state(harness, position, button, false);
    wait_for_html_browser_frame_advance_and_idle(
        harness,
        recording,
        hover_generation,
        timeout_seconds,
    )
}

fn current_html_browser_frame_generation(harness: &mut Harness<'_, KatanaApp>) -> Result<u64> {
    harness
        .state_mut()
        .html_browser_frame_generation_for_test()
        .context("wait_for_html_frame requires an active HTML browser frame")
}

fn click_staging_position(display_rect: egui::Rect, target: egui::Pos2) -> egui::Pos2 {
    let center = display_rect.center();
    let x = if target.x <= center.x {
        display_rect.right() - 1.0
    } else {
        display_rect.left() + 1.0
    };
    let y = if target.y <= center.y {
        display_rect.bottom() - 1.0
    } else {
        display_rect.top() + 1.0
    };
    egui::pos2(x, y)
}

fn html_viewport_fraction_position(
    harness: &mut Harness<'_, KatanaApp>,
    x_fraction: f32,
    y_fraction: f32,
) -> Result<egui::Pos2> {
    ensure!(
        (0.0..=1.0).contains(&x_fraction) && (0.0..=1.0).contains(&y_fraction),
        "HTML viewport click fractions must be between 0 and 1"
    );
    let display_rect = harness
        .state_mut()
        .html_browser_display_rect_for_test()
        .context("HTML viewport click requires an active display rect")?;
    Ok(egui::pos2(
        display_rect.left() + display_rect.width() * x_fraction,
        display_rect.top() + display_rect.height() * y_fraction,
    ))
}

fn wait_for_html_browser_frame(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    previous_frame: Option<HtmlBrowserFrameIdentity>,
    timeout_seconds: f64,
) -> Result<Duration> {
    let started_at = Instant::now();
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        let current_frame = html_browser_frame_identity(harness);
        if html_browser_frame_advanced(previous_frame.as_ref(), current_frame.as_ref()) {
            let elapsed = started_at.elapsed();
            println!(
                "  HTML browser first frame ready in {:.3}s",
                elapsed.as_secs_f64()
            );
            return Ok(elapsed);
        }
        if Instant::now() >= deadline {
            bail!("HTML browser did not produce an initial frame within {timeout_seconds:.2}s");
        }
        sleep_frame(60.0);
    }
}

fn assert_first_frame_latency(
    surface: &str,
    elapsed: Duration,
    maximum_seconds: Option<f64>,
) -> Result<()> {
    let Some(maximum_seconds) = maximum_seconds else {
        return Ok(());
    };
    ensure!(
        maximum_seconds.is_finite() && maximum_seconds > 0.0,
        "max_first_frame_seconds must be a positive finite number"
    );
    ensure!(
        elapsed.as_secs_f64() <= maximum_seconds,
        "{surface} first frame took {:.3}s, exceeding the {:.3}s regression limit",
        elapsed.as_secs_f64(),
        maximum_seconds
    );
    Ok(())
}

fn opened_url_frame_ready(
    previous_frame: Option<&HtmlBrowserFrameIdentity>,
    current_frame: Option<&HtmlBrowserFrameIdentity>,
    is_loading: bool,
    active_source_url: &str,
) -> bool {
    !is_loading
        && current_frame.is_some_and(|current| current.origin == active_source_url)
        && html_browser_frame_advanced(previous_frame, current_frame)
}

fn open_url_and_wait_for_html_frame(
    harness: &mut Harness<'_, KatanaApp>,
    mut recording: Option<&mut ActiveRecording>,
    url: &str,
    timeout_seconds: f64,
    expected_error_contains: Option<&str>,
) -> Result<()> {
    let previous_frame = html_browser_frame_identity(harness);
    harness
        .state_mut()
        .trigger_action(AppAction::OpenUrl(url.to_string()));
    harness.ctx.request_repaint();
    let deadline = async_assert_deadline(timeout_seconds)?;
    loop {
        harness.step();
        maybe_capture_recording_frame(harness, recording.as_deref_mut())?;
        let (error, is_loading, active_source_url) = {
            let app = harness.state_mut();
            (
                app.app_state_for_test().url_tab.last_error.clone(),
                app.app_state_for_test().url_tab.is_loading,
                app.app_state_for_test().url_tab.input.clone(),
            )
        };
        if let Some(error) = error {
            if let Some(expected) = expected_error_contains {
                let error = error.to_string();
                ensure!(
                    error.contains(expected),
                    "opening URL {url:?} failed with {error:?}, expected diagnostic containing {expected:?}"
                );
                println!("  URL produced expected diagnostic: {error}");
                return Ok(());
            }
            bail!("opening URL {url:?} failed: {error}");
        }
        let current_frame = html_browser_frame_identity(harness);
        if opened_url_frame_ready(
            previous_frame.as_ref(),
            current_frame.as_ref(),
            is_loading,
            &active_source_url,
        ) {
            ensure!(
                expected_error_contains.is_none(),
                "URL {url:?} produced an HTML frame, expected an error containing {:?}",
                expected_error_contains.unwrap_or_default()
            );
            let current_frame = current_frame.expect("ready frame was present");
            println!(
                "  URL produced HTML frame: origin={:?}, generation={}",
                current_frame.origin, current_frame.generation
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "URL {url:?} did not produce its HTML frame within {timeout_seconds:.2}s; \
                 active source was {active_source_url:?}, last frame was {current_frame:?}, \
                 loading was {is_loading}"
            );
        }
        sleep_frame(60.0);
    }
}

fn sleep_frame(fps: f64) {
    std::thread::sleep(Duration::from_secs_f64(1.0 / fps.max(1.0)));
}

fn parse_settings_tab(tab: &str) -> (SettingsTab, SettingsSection) {
    let t = match tab {
        "theme" => SettingsTab::Theme,
        "icons" => SettingsTab::Icons,
        "font" => SettingsTab::Font,
        "layout" => SettingsTab::Layout,
        "workspace" => SettingsTab::Workspace,
        "updates" => SettingsTab::Updates,
        "behavior" => SettingsTab::Behavior,
        "shortcuts" => SettingsTab::Shortcuts,
        "linter" => SettingsTab::Linter,
        other => {
            println!("  WARNING: unknown settings tab {other:?}, defaulting to theme");
            SettingsTab::Theme
        }
    };
    let s = t.section();
    (t, s)
}

fn first_file_in_tree(tree: &[TreeEntry]) -> Option<PathBuf> {
    for entry in tree {
        match entry {
            TreeEntry::File { path } => return Some(path.clone()),
            TreeEntry::Directory { children, .. } => {
                if let Some(p) = first_file_in_tree(children) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn find_workspace_file(
    tree: &[TreeEntry],
    workspace_root: Option<&Path>,
    requested: &str,
) -> Option<PathBuf> {
    if let Some(root) = workspace_root {
        if let Some(path) = find_file_by_relative_path(tree, root, Path::new(requested)) {
            return Some(path);
        }
    }
    find_file_by_name(tree, requested)
}

fn find_file_by_relative_path(
    tree: &[TreeEntry],
    workspace_root: &Path,
    requested: &Path,
) -> Option<PathBuf> {
    let requested = normalize_relative_path(requested);
    for entry in tree {
        match entry {
            TreeEntry::File { path } => {
                let Ok(relative) = path.strip_prefix(workspace_root) else {
                    continue;
                };
                if normalize_relative_path(relative) == requested {
                    return Some(path.clone());
                }
            }
            TreeEntry::Directory { children, .. } => {
                if let Some(path) = find_file_by_relative_path(children, workspace_root, &requested)
                {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn find_file_by_name(tree: &[TreeEntry], name: &str) -> Option<PathBuf> {
    for entry in tree {
        match entry {
            TreeEntry::File { path } => {
                if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                    return Some(path.clone());
                }
            }
            TreeEntry::Directory { children, .. } => {
                if let Some(p) = find_file_by_name(children, name) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    #[test]
    fn node_region_rejects_displaced_clipped_and_invalid_bounds() {
        let viewport = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1280.0, 900.0));
        let tab = egui::Rect::from_min_max(egui::pos2(349.7, 839.0), egui::pos2(398.5, 862.0));
        let bounds = [250.0, 828.0, 1250.0, 872.0];
        assert!(super::assert_node_bounds(tab, viewport, bounds).is_ok());
        assert!(
            super::assert_node_bounds(tab.translate(egui::vec2(0.0, -100.0)), viewport, bounds)
                .is_err()
        );
        assert!(
            super::assert_node_bounds(tab.translate(egui::vec2(0.0, 20.0)), viewport, bounds)
                .is_err()
        );
        for invalid in [
            [250.0, 828.0, 250.0, 872.0],
            [1250.0, 828.0, 250.0, 872.0],
            [0.0, 0.0, 1281.0, 900.0],
            [f32::NAN, 828.0, 1250.0, 872.0],
        ] {
            assert!(super::assert_node_bounds(tab, viewport, invalid).is_err());
        }
    }
    #[test]
    fn accessibility_click_uses_logical_coordinates_at_high_dpi() {
        for scale in [1.0, 2.0] {
            let mut harness = egui_kittest::Harness::builder()
                .with_size(egui::vec2(400.0, 240.0))
                .with_pixels_per_point(scale)
                .build_ui_state(
                    |ui, clicked| {
                        let response = ui.put(
                            egui::Rect::from_min_size(
                                egui::pos2(160.0, 100.0),
                                egui::vec2(120.0, 32.0),
                            ),
                            egui::Button::new("Coordinate target"),
                        );
                        if response.clicked() {
                            *clicked = true;
                        }
                    },
                    false,
                );
            harness.run();
            super::click_node(
                &mut harness,
                "Coordinate target",
                crate::request::ClickButton::Primary,
            );
            assert!(
                *harness.state(),
                "click missed the real button at scale {scale}"
            );
        }
    }
    use super::{
        HtmlBrowserFrameIdentity, assert_first_frame_latency, click_staging_position,
        document_failure_advanced, find_workspace_file, html_browser_frame_advanced,
        normalize_relative_path, opened_url_frame_ready, physical_png_bounds, scroll_delta,
    };
    use crate::capture::PngBounds;
    use crate::request::ScrollDirection;
    use katana_core::workspace::TreeEntry;
    use std::{
        path::{Path, PathBuf},
        time::Duration,
    };

    #[test]
    fn open_file_prefers_workspace_relative_path_over_basename_match() {
        let root = Path::new("/tmp/workspace");
        let tree = vec![
            TreeEntry::Directory {
                path: root.join("openspec"),
                children: vec![TreeEntry::File {
                    path: root.join("openspec/README.md"),
                }],
            },
            TreeEntry::File {
                path: root.join("README.md"),
            },
        ];

        let resolved = find_workspace_file(&tree, Some(root), "README.md");

        assert_eq!(resolved, Some(root.join("README.md")));
    }

    #[test]
    fn open_file_supports_nested_relative_paths() {
        let root = Path::new("/tmp/workspace");
        let tree = vec![TreeEntry::Directory {
            path: root.join("openspec"),
            children: vec![TreeEntry::File {
                path: root.join("openspec/README.md"),
            }],
        }];

        let resolved = find_workspace_file(&tree, Some(root), "openspec/README.md");

        assert_eq!(resolved, Some(root.join("openspec/README.md")));
    }

    #[test]
    fn browser_frame_wait_requires_a_new_document_or_generation() {
        let previous = HtmlBrowserFrameIdentity {
            document_path: PathBuf::from("/tmp/first.html"),
            origin: "file:///tmp/first.html".to_string(),
            generation: 1,
        };
        let advanced = HtmlBrowserFrameIdentity {
            generation: 2,
            ..previous.clone()
        };
        let different_document = HtmlBrowserFrameIdentity {
            document_path: PathBuf::from("/tmp/second.html"),
            origin: "file:///tmp/second.html".to_string(),
            generation: 1,
        };

        assert!(html_browser_frame_advanced(None, Some(&previous)));
        assert!(!html_browser_frame_advanced(
            Some(&previous),
            Some(&previous)
        ));
        assert!(html_browser_frame_advanced(
            Some(&previous),
            Some(&advanced)
        ));
        assert!(html_browser_frame_advanced(
            Some(&previous),
            Some(&different_document)
        ));
        assert!(!html_browser_frame_advanced(Some(&previous), None));
    }

    #[test]
    fn document_frame_wait_ignores_only_the_previous_failure() {
        assert!(!document_failure_advanced(None, None));
        assert!(!document_failure_advanced(Some("old"), Some("old")));
        assert!(document_failure_advanced(None, Some("new")));
        assert!(document_failure_advanced(Some("old"), Some("new")));
    }

    #[test]
    fn opened_url_frame_accepts_a_final_redirect_origin_only_after_loading() {
        let previous = HtmlBrowserFrameIdentity {
            document_path: PathBuf::from("Katana://URL/start.html"),
            origin: "https://example.test/start".to_string(),
            generation: 3,
        };
        let redirected = HtmlBrowserFrameIdentity {
            document_path: PathBuf::from("Katana://URL/final.html"),
            origin: "https://example.test/final".to_string(),
            generation: 1,
        };
        let animated_previous = HtmlBrowserFrameIdentity {
            generation: 4,
            ..previous.clone()
        };

        assert!(!opened_url_frame_ready(
            Some(&previous),
            Some(&animated_previous),
            true,
            "https://example.test/start"
        ));
        assert!(!opened_url_frame_ready(
            Some(&previous),
            Some(&redirected),
            false,
            "https://example.test/start"
        ));
        assert!(opened_url_frame_ready(
            Some(&previous),
            Some(&redirected),
            false,
            "https://example.test/final"
        ));
    }

    #[test]
    fn normalize_relative_path_collapses_dot_segments() {
        assert_eq!(
            normalize_relative_path(Path::new("./docs/../README.md")),
            PathBuf::from("README.md"),
        );
    }

    #[test]
    fn physical_png_bounds_scale_and_clip_the_html_surface() {
        let rect = egui::Rect::from_min_max(egui::pos2(-10.0, 20.0), egui::pos2(120.0, 90.0));

        assert_eq!(
            physical_png_bounds(rect, 200, 100, 2.0),
            Some(PngBounds {
                x: 0,
                y: 40,
                width: 200,
                height: 60,
            })
        );
    }

    #[test]
    fn physical_png_bounds_reject_non_visible_or_invalid_rectangles() {
        let outside = egui::Rect::from_min_max(egui::pos2(150.0, 150.0), egui::pos2(200.0, 200.0));

        assert_eq!(physical_png_bounds(outside, 200, 100, 2.0), None);
        assert_eq!(physical_png_bounds(egui::Rect::NAN, 200, 100, 2.0), None);
        assert_eq!(
            physical_png_bounds(egui::Rect::EVERYTHING, 200, 100, 0.0),
            None
        );
    }

    #[test]
    fn scroll_delta_maps_every_direction_to_one_axis() {
        assert_eq!(scroll_delta(ScrollDirection::Up, 8.0), egui::vec2(0.0, 8.0));
        assert_eq!(
            scroll_delta(ScrollDirection::Down, 8.0),
            egui::vec2(0.0, -8.0)
        );
        assert_eq!(
            scroll_delta(ScrollDirection::Left, 8.0),
            egui::vec2(8.0, 0.0)
        );
        assert_eq!(
            scroll_delta(ScrollDirection::Right, 8.0),
            egui::vec2(-8.0, 0.0)
        );
    }

    #[test]
    fn click_staging_position_uses_the_opposite_display_quadrant() {
        let rect = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(110.0, 220.0));

        assert_eq!(
            click_staging_position(rect, egui::pos2(20.0, 30.0)),
            egui::pos2(109.0, 219.0)
        );
        assert_eq!(
            click_staging_position(rect, egui::pos2(100.0, 200.0)),
            egui::pos2(11.0, 21.0)
        );
    }

    #[test]
    fn first_frame_latency_limit_rejects_slow_or_invalid_measurements() {
        assert!(assert_first_frame_latency("document", Duration::from_secs(1), None).is_ok());
        assert!(assert_first_frame_latency("document", Duration::from_secs(1), Some(2.0)).is_ok());
        assert!(assert_first_frame_latency("document", Duration::from_secs(3), Some(2.0)).is_err());
        assert!(assert_first_frame_latency("document", Duration::ZERO, Some(f64::NAN)).is_err());
        assert!(assert_first_frame_latency("document", Duration::ZERO, Some(0.0)).is_err());
    }
}
