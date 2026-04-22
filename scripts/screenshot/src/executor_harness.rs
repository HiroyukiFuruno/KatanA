use crate::request::{Fixture, Step, UiAction};
use anyhow::Result;
use egui_kittest::Harness;
use katana_core::workspace::TreeEntry;
use katana_ui::app_state::{AppAction, AppState, SettingsSection, SettingsTab};
use katana_ui::shell::KatanaApp;
use std::path::{Path, PathBuf};

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

    for (i, step) in steps.iter().enumerate() {
        let label = match step {
            Step::Launch(_) => "launch",
            Step::Wait(_) => "wait",
            Step::Screenshot(_) => "screenshot",
            Step::OpenFile(_) => "open_file",
            Step::Action(_) => "action",
            Step::Quit => "quit",
        };
        println!("step {}/{}: {label}", i + 1, steps.len());

        match step {
            Step::Launch(s) => {
                let frames = ((s.wait_seconds * 60.0) as usize).max(30);
                for _ in 0..frames {
                    harness.step();
                }
                for _ in 0..200 {
                    harness.step();
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
                    for _ in 0..60 {
                        harness.step();
                    }
                }
            }
            Step::Wait(s) => {
                let frames = ((s.seconds * 60.0) as usize).max(1);
                for _ in 0..frames {
                    harness.step();
                }
            }
            Step::Screenshot(s) => {
                harness.run_steps(120);
                let image = harness
                    .render()
                    .map_err(|e| anyhow::anyhow!("render failed: {e}"))?;
                let out = output_dir.join(format!("{}.png", s.output_name));
                image
                    .save(&out)
                    .map_err(|e| anyhow::anyhow!("save failed: {e}"))?;
                println!("  saved: {}", out.display());
            }
            Step::OpenFile(s) => {
                let path = harness
                    .state_mut()
                    .app_state_mut()
                    .workspace
                    .data
                    .as_ref()
                    .and_then(|ws| find_file_by_name(&ws.tree, &s.file_name));
                match path {
                    Some(p) => {
                        harness.state_mut().trigger_action(AppAction::SelectDocument(p));
                        let frames = ((s.wait_seconds * 60.0) as usize).max(30);
                        for _ in 0..frames {
                            harness.step();
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
                        harness.state_mut().trigger_action(AppAction::ToggleSettings);
                        for _ in 0..30 {
                            harness.step();
                        }
                        let (settings_tab, settings_section) = parse_settings_tab(tab);
                        let config = &mut harness.state_mut().app_state_mut().config;
                        config.active_settings_tab = settings_tab;
                        config.active_settings_section = settings_section;
                        for _ in 0..60 {
                            harness.step();
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
                        for _ in 0..60 {
                            harness.step();
                        }
                    }
                    UiAction::OpenIconsAdvancedPanel => {
                        harness.ctx.data_mut(|d| {
                            d.insert_temp(egui::Id::new("icons_advanced_is_open"), true);
                        });
                        for _ in 0..60 {
                            harness.step();
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
                        for _ in 0..60 {
                            harness.step();
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
                        for _ in 0..60 {
                            harness.step();
                        }
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
                            UiAction::ToggleExportPanel => AppAction::ToggleExportPanel,
                            UiAction::OpenChangelog => AppAction::ShowReleaseNotes,
                            UiAction::OpenSettingsTab { .. }
                            | UiAction::ForceOpenAccordion { .. }
                            | UiAction::OpenIconsAdvancedPanel
                            | UiAction::ScrollDown { .. }
                            | UiAction::SetViewMode { .. } => unreachable!(),
                        };
                        harness.state_mut().trigger_action(app_action);
                        for _ in 0..60 {
                            harness.step();
                        }
                    }
                }
            }
            Step::Quit => {}
        }
    }

    Ok(())
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
