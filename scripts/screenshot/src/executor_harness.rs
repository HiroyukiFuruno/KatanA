use crate::request::{ClickButton, Fixture, ScrollDirection, Step, UiAction, VideoFormat};
use anyhow::{bail, Context, Result};
use egui_kittest::{kittest::Queryable, Harness};
use katana_core::workspace::TreeEntry;
use katana_ui::app_state::{AppAction, AppState, SettingsSection, SettingsTab};
use katana_ui::shell::KatanaApp;
use katana_ui::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
};
use katana_ui::state::command_palette_providers::{
    AppCommandProvider, MarkdownContentProvider, WorkspaceFileProvider,
};
use katana_platform::theme::{ThemeMode, ThemePreset};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

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
    let (width, height) = steps
        .iter()
        .find_map(|s| {
            if let Step::Launch(ls) = s {
                ls.viewport.map(|v| (v.width as f32, v.height as f32))
            } else {
                None
            }
        })
        .unwrap_or((1728.0, 1117.0));

    let locale = fixture.settings.locale.as_deref().unwrap_or("en");
    katana_ui::i18n::I18nOps::set_language(locale);

    let settings_path = config_dir.join("settings.json");
    let workspace_dir_owned = workspace_dir.map(|p| p.to_path_buf());
    let workspace_dir_for_lookup = workspace_dir_owned.clone();
    let output_dir = output_dir.to_path_buf();

    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .with_pixels_per_point(2.0)
        .build_eframe(move |cc| {
            use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
            use katana_platform::SettingsService;

            let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
            katana_ui::font_loader::SystemFontLoader::setup_fonts(&cc.egui_ctx, &preset, None, None);
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
            state
                .config
                .settings
                .settings_mut()
                .terms_accepted_version =
                Some(katana_ui::about_info::APP_VERSION.to_string());
            state
                .config
                .settings
                .settings_mut()
                .updates
                .previous_app_version =
                Some(katana_ui::about_info::APP_VERSION.to_string());
            state.global_workspace =
                katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
                    katana_platform::workspace::InMemoryWorkspaceRepository::default(),
                ));
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
    let mut recording: Option<ActiveRecording> = None;

    for (i, step) in steps.iter().enumerate() {
        let label = match step {
            Step::Launch(_) => "launch",
            Step::Wait(_) => "wait",
            Step::Screenshot(_) => "screenshot",
            Step::RecordStart(_) => "record_start",
            Step::RecordStop(_) => "record_stop",
            Step::Scroll(_) => "scroll",
            Step::ExportPng(_) => "export_png",
            Step::OpenFile(_) => "open_file",
            Step::Action(_) => "action",
            Step::Quit => "quit",
        };
        println!("step {}/{}: {label}", i + 1, steps.len());

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
                let first = harness
                    .state_mut()
                    .app_state_mut()
                    .workspace
                    .data
                    .as_ref()
                    .and_then(|ws| first_file_in_tree(&ws.tree));
                if let Some(path) = first {
                    harness.state_mut().trigger_action(AppAction::SelectDocument(path));
                    let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
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
                let out = output_dir.join(format!(
                    "{}.{}",
                    recorder.output_name,
                    recorder.extension()
                ));
                encode_video(&recorder, &out)?;
                println!("  recorded: {}", out.display());
            }
            Step::Scroll(s) => {
                let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                let frames = ((s.duration_seconds * fps) as usize).max(1);
                let delta_per_frame = s.pixels / frames as f32;
                for _ in 0..frames {
                    let viewport = harness.ctx.viewport_rect();
                    let pos = egui::pos2(viewport.center().x, viewport.center().y);
                    let signed_delta = match s.direction {
                        ScrollDirection::Down => -delta_per_frame,
                        ScrollDirection::Up => delta_per_frame,
                    };
                    harness.input_mut().events.push(egui::Event::PointerMoved(pos));
                    harness.input_mut().events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::Vec2::new(0.0, signed_delta),
                        modifiers: egui::Modifiers::NONE,
                        phase: egui::TouchPhase::Move,
                    });
                    harness.step();
                    maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                    sleep_frame(fps);
                }
            }
            Step::ExportPng(s) => {
                // Get the active document's markdown content and path
                let doc_info = harness
                    .state_mut()
                    .app_state_mut()
                    .active_document()
                    .map(|d| (d.buffer.clone(), d.path.clone()));
                let (source, doc_path) = match doc_info {
                    Some(info) => info,
                    None => {
                        println!("  WARNING: no active document for export_png, skipping");
                        continue;
                    }
                };
                let preset = katana_core::markdown::color_preset::DiagramColorPreset::current().clone();
                let base_dir = doc_path.parent().map(|p| p.to_path_buf());
                let tmp_html_name = format!("katana_screenshot_export_{}.html", s.output_name);
                let html_path = katana_ui::shell_logic::ShellLogicOps::export_named_html_to_tmp(
                    &source,
                    &tmp_html_name,
                    &preset,
                    base_dir.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!("html export failed: {e}"))?;
                let out = output_dir.join(format!("{}.png", s.output_name));
                katana_core::markdown::export::ImageExporter::export(
                    &std::fs::read_to_string(&html_path)
                        .map_err(|e| anyhow::anyhow!("read html failed: {e}"))?,
                    &out,
                )
                .map_err(|e| anyhow::anyhow!("png export failed: {e}"))?;
                let _ = std::fs::remove_file(&html_path);
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
                        harness.state_mut().trigger_action(AppAction::SelectDocument(p));
                        let fps = recording.as_ref().map(|r| r.fps as f64).unwrap_or(60.0);
                        let frames = ((s.wait_seconds * fps) as usize).max(30);
                        for _ in 0..frames {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    None => {
                        println!("  WARNING: file {:?} not found in workspace tree", s.file_name);
                    }
                }
            }
            Step::Action(a) => {
                match &a.action {
                    UiAction::OpenSettingsTab { tab } => {
                        if !harness.state_mut().app_state_mut().layout.show_settings {
                            harness.state_mut().trigger_action(AppAction::ToggleSettings);
                        }
                        for _ in 0..30 {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                        let (settings_tab, settings_section) = parse_settings_tab(tab);
                        let config = &mut harness.state_mut().app_state_mut().config;
                        config.active_settings_tab = settings_tab;
                        config.active_settings_section = settings_section;
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::ForceOpenAccordion { id } => {
                        let egui_id = egui::Id::new(id.as_str());
                        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                            &harness.ctx,
                            egui_id,
                            false,
                        );
                        state.set_open(true);
                        state.store(&harness.ctx);
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::OpenIconsAdvancedPanel => {
                        harness.ctx.data_mut(|d| {
                            d.insert_temp(egui::Id::new("icons_advanced_is_open"), true);
                        });
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                    UiAction::ScrollDown { amount } => {
                        // Move pointer to center of settings content area, then scroll
                        let viewport = harness.ctx.viewport_rect();
                        let pos = egui::pos2(viewport.center().x, viewport.center().y);
                        harness.input_mut().events.push(egui::Event::PointerMoved(pos));
                        harness.input_mut().events.push(egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::Vec2::new(0.0, *amount),
                            modifiers: egui::Modifiers::NONE,
                            phase: egui::TouchPhase::Move,
                        });
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
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
                            harness.input_mut().events.push(egui::Event::PointerMoved(pos));
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
                        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                            &harness.ctx,
                            egui_id,
                            false,
                        );
                        state.set_open(true);
                        state.store(&harness.ctx);
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
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
                                println!("  WARNING: unknown view mode {other:?}, defaulting to preview_only");
                                ViewMode::PreviewOnly
                            }
                        };
                        harness.state_mut().trigger_action(AppAction::SetViewMode(view_mode));
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
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
                        select_theme_preset_in_settings(
                            &mut harness,
                            recording.as_mut(),
                            preset,
                        )?;
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
                    UiAction::ClickNode { label, button, wait_seconds } => {
                        click_node(&mut harness, label, *button);
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    UiAction::ClickAt { x, y, button, wait_seconds } => {
                        click_at(&mut harness, egui::pos2(*x, *y), *button);
                        step_for_seconds(&mut harness, recording.as_mut(), *wait_seconds)?;
                    }
                    other => {
                        let app_action = match other {
                            UiAction::ToggleToc => AppAction::ToggleToc,
                            UiAction::ToggleSplitView => AppAction::SetViewMode(
                                katana_ui::app_state::ViewMode::Split,
                            ),
                            UiAction::ToggleSettings => AppAction::ToggleSettings,
                            UiAction::ToggleExplorer => AppAction::ToggleExplorer,
                            UiAction::ToggleSlideshow => AppAction::ToggleSlideshow,
                            UiAction::ToggleStoryPanel => AppAction::ToggleStoryPanel,
                            UiAction::ToggleExportPanel => AppAction::ToggleExportPanel,
                            UiAction::OpenChangelog => AppAction::ShowReleaseNotes,
                            UiAction::OpenHelpDemo => AppAction::OpenHelpDemo,
                            UiAction::SelectNextTab => AppAction::SelectNextTab,
                            UiAction::OpenSettingsTab { .. }
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
                            | UiAction::ClickNode { .. }
                            | UiAction::ClickAt { .. } => unreachable!(),
                        };
                        harness.state_mut().trigger_action(app_action);
                        let fps = recording.as_ref().map(|r| r.fps as u32).unwrap_or(60);
                        for _ in 0..fps {
                            harness.step();
                            maybe_capture_recording_frame(&mut harness, recording.as_mut())?;
                        }
                    }
                }
            }
            Step::Quit => {}
        }
    }
    if recording.is_some() {
        bail!("record_start was called but record_stop was not reached");
    }

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
    let mut cmd = Command::new("ffmpeg");
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
        step_for_seconds(harness, recording.as_deref_mut(), 0.45)?;
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
        app.command_palette.current_query[1..].trim_start().to_string()
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
    harness.state_mut().trigger_action(AppAction::ToggleSearchModal);
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
    step_for_seconds(harness, recording.as_deref_mut(), pause_after_seconds)?;
    Ok(())
}

fn apply_global_search_query(harness: &mut Harness<'_, KatanaApp>, tab: &str, query: &str) {
    let app = harness.state_mut().app_state_mut();
    if tab == "markdown_content" {
        app.search.md_search.query = query.to_string();
        app.search.md_last_params = Some(app.search.md_search.clone());
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
        harness.state_mut().trigger_action(AppAction::DocSearchQueryChanged);
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

fn click_node(harness: &mut Harness<'_, KatanaApp>, label: &str, button: ClickButton) {
    let node = harness.get_by_label(label);
    match button {
        ClickButton::Primary => node.click(),
        ClickButton::Secondary => node.click_secondary(),
    }
}

fn click_at(harness: &mut Harness<'_, KatanaApp>, pos: egui::Pos2, button: ClickButton) {
    let pointer_button = match button {
        ClickButton::Primary => egui::PointerButton::Primary,
        ClickButton::Secondary => egui::PointerButton::Secondary,
    };
    harness.input_mut().events.push(egui::Event::PointerMoved(pos));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos,
        button: pointer_button,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos,
        button: pointer_button,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
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
    use super::{find_workspace_file, normalize_relative_path};
    use katana_core::workspace::TreeEntry;
    use std::path::{Path, PathBuf};

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
    fn normalize_relative_path_collapses_dot_segments() {
        assert_eq!(
            normalize_relative_path(Path::new("./docs/../README.md")),
            PathBuf::from("README.md"),
        );
    }
}
