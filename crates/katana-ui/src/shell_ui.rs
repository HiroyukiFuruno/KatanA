use crate::app::*;
use crate::app_state::AppAction;
use eframe::egui;

const INVISIBLE_LABEL_SIZE: f32 = 0.1;

pub struct ShellUiOps;

impl ShellUiOps {
    pub(crate) fn invisible_label(text: &str) -> egui::RichText {
        egui::RichText::new(text)
            .size(INVISIBLE_LABEL_SIZE)
            .color(crate::theme_bridge::TRANSPARENT)
    }

    pub(crate) fn indent_prefix(depth: usize) -> String {
        "  ".repeat(depth)
    }

    pub fn update_native_menu_strings_from_i18n() {
        crate::native_menu::update_native_menu_strings_from_i18n();
    }

    pub(crate) fn relative_full_path(
        path: &std::path::Path,
        ws_root: Option<&std::path::Path>,
    ) -> String {
        crate::shell_logic::ShellLogicOps::relative_full_path(path, ws_root)
    }

    pub(crate) fn open_folder_dialog() -> Option<std::path::PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }
}

pub(crate) const WORKSPACE_SPINNER_OUTER_MARGIN: f32 = 10.0;
pub(crate) const WORKSPACE_SPINNER_INNER_MARGIN: f32 = 10.0;
pub(crate) const WORKSPACE_SPINNER_TEXT_MARGIN: f32 = 5.0;
pub(crate) const STATUS_SUCCESS_GREEN: u8 = 200;
pub(crate) const STATUS_BAR_ICON_SPACING: f32 = 4.0;

pub(crate) const SEARCH_MODAL_WIDTH: f32 = 500.0;
pub(crate) const SEARCH_MODAL_HEIGHT: f32 = 400.0;
pub(crate) const TOC_PANEL_DEFAULT_WIDTH: f32 = 200.0;
pub(crate) const TOC_PANEL_MARGIN: f32 = 8.0;
pub(crate) const TOC_HEADING_VISIBILITY_THRESHOLD: f32 = 40.0;
pub(crate) const TOC_INDENT_PER_LEVEL: f32 = 12.0;
pub(crate) const LIGHT_MODE_ICON_BG: u8 = 235;
pub(crate) const LIGHT_MODE_ICON_ACTIVE_BG: u8 = 200;

pub(crate) struct TreeRenderContext<'a, 'b> {
    pub action: &'a mut AppAction,
    pub depth: usize,
    pub active_path: Option<&'b std::path::Path>,
    pub filter_set: Option<&'b std::collections::HashSet<std::path::PathBuf>>,
    pub expanded_directories: &'a mut std::collections::HashSet<std::path::PathBuf>,
    pub disable_context_menu: bool,
    pub is_flat_view: bool,
    pub ws_root: Option<&'b std::path::Path>,
}

#[cfg(all(target_os = "macos", not(test)))]
pub use crate::native_menu::{native_menu_setup, native_set_app_icon_png, native_set_process_name};
use crate::shell::KatanaApp;

pub(crate) const SPLIT_HALF_RATIO: f32 = 0.5;
pub(crate) const SPLIT_PANEL_MAX_RATIO: f32 = 0.7;
pub(crate) const PREVIEW_CONTENT_PADDING: i8 = 12;

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.needs_splash {
            self.splash_start = Some(std::time::Instant::now());
            self.needs_splash = false;
        }

        let auto_save_enabled = self.state.config.settings.settings().behavior.auto_save;
        let auto_save_interval = self
            .state
            .config
            .settings
            .settings()
            .behavior
            .auto_save_interval_secs;
        if auto_save_enabled && auto_save_interval > 0.0 {
            let now = std::time::Instant::now();
            if let Some(last) = self.state.document.last_auto_save {
                if now.duration_since(last).as_secs_f64() >= auto_save_interval {
                    if let Some(doc) = self.state.active_document()
                        && doc.is_dirty
                    {
                        self.pending_action = crate::app_state::AppAction::SaveDocument;
                    }
                    self.state.document.last_auto_save = Some(now);
                }
            } else {
                self.state.document.last_auto_save = Some(now);
            }
        } else {
            self.state.document.last_auto_save = None;
        }

        let auto_refresh_enabled = self.state.config.settings.settings().behavior.auto_refresh;
        let auto_refresh_interval = self
            .state
            .config
            .settings
            .settings()
            .behavior
            .auto_refresh_interval_secs;
        if auto_refresh_enabled && auto_refresh_interval > 0.0 {
            let now = std::time::Instant::now();
            if let Some(last) = self.state.document.last_auto_refresh {
                if now.duration_since(last).as_secs_f64() >= auto_refresh_interval {
                    if self.state.active_document().is_some() {
                        self.pending_action =
                            crate::app_state::AppAction::RefreshDocument { is_manual: false };
                    }
                    self.state.document.last_auto_refresh = Some(now);
                }
            } else {
                self.state.document.last_auto_refresh = Some(now);
            }
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(auto_refresh_interval));
        } else {
            self.state.document.last_auto_refresh = None;
        }

        let theme_colors = self
            .state
            .config
            .settings
            .settings()
            .effective_theme_colors();
        if self.cached_theme.as_ref() != Some(&theme_colors) {
            let dark = theme_colors.mode == katana_platform::theme::ThemeMode::Dark;
            ctx.set_visuals(crate::theme_bridge::visuals_from_theme(&theme_colors));
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("katana_theme_colors"), theme_colors.clone());
            });
            ctx.global_style_mut(|s| s.spacing.scroll.floating = false);
            katana_core::markdown::color_preset::DiagramColorPreset::set_dark_mode(dark);
            self.cached_theme = Some(theme_colors.clone());
            if matches!(self.pending_action, AppAction::None) {
                self.pending_action = AppAction::RefreshDiagrams;
            }
        }

        let font_size = self.state.config.settings.settings().clamped_font_size();
        if self.cached_font_size != Some(font_size) {
            crate::theme_bridge::apply_font_size(ctx, font_size);
            self.cached_font_size = Some(font_size);
        }

        let font_family = self.state.config.settings.settings().font.family.clone();
        if self.cached_font_family.as_deref() != Some(&font_family) {
            crate::theme_bridge::apply_font_family(ctx, &font_family);
            self.cached_font_family = Some(font_family);
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers {
                    command: true,
                    shift: true,
                    ..Default::default()
                },
                egui::Key::T,
            ))
        }) && !self.state.document.recently_closed_tabs.is_empty()
        {
            self.pending_action = AppAction::RestoreClosedDocument;
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::P,
            )) || i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::P,
            )) || i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::K,
            ))
        }) {
            self.pending_action = AppAction::ToggleCommandPalette;
        }

        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::F,
            ))
        }) {
            if !self.state.search.doc_search_open {
                self.state.search.doc_search_open = true;
                self.trigger_action(AppAction::DocSearchQueryChanged);
            } else {
                self.state.search.doc_search_open = false;
                self.state.search.doc_search_matches.clear();
            }
        }

        self.poll_download(ctx);
        self.poll_workspace_load(ctx);

        if let Some(path) = self.pending_document_loads.pop_front() {
            self.handle_select_document(path, false);
            ctx.request_repaint();
        }

        self.poll_update_install(ctx);
        self.poll_update_check(ctx);
        self.poll_changelog(ctx);
        self.poll_export(ctx);

        let native_action = crate::native_menu::poll_native_menu(
            &mut self.show_about,
            ShellUiOps::open_folder_dialog,
        );
        if !matches!(native_action, AppAction::None) {
            self.pending_action = native_action;
        }

        let action = self.take_action();
        crate::views::panels::preview::PreviewPanelOps::invalidate_preview_image_cache(
            ctx, &action,
        );
        self.process_action(ctx, action);

        let terms_ver = crate::about_info::APP_VERSION.to_string();
        let accepted_ver = self
            .state
            .config
            .settings
            .settings()
            .terms_accepted_version
            .as_ref();
        if accepted_ver != Some(&terms_ver) {
            egui::CentralPanel::default().show(ctx, |ui| {
                crate::views::modals::terms::TermsModal::new(&terms_ver, &mut self.pending_action)
                    .show(ui);
            });
            return;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                let download_req =
                    crate::views::app_frame::MainPanels::new(self, &theme_colors).show(ui);
                if let Some(req) = download_req {
                    self.start_download(req);
                }
            });

        if self.state.layout.show_slideshow {
            if let Some(doc) = self.state.active_document() {
                if let Some(preview) = self.tab_previews.iter_mut().find(|p| p.path == doc.path) {
                    crate::preview_pane::fullscreen::render_slideshow_modal(
                        ctx,
                        &mut self.state.layout,
                        &mut preview.pane,
                    );
                } else {
                    self.state.layout.show_slideshow = false;
                }
            } else {
                self.state.layout.show_slideshow = false;
            }
        }

        if let Some(settings_action) =
            crate::settings::SettingsWindow::new(&mut self.state, &mut self.settings_preview)
                .show(ctx)
        {
            self.pending_action = settings_action;
        }

        if self.state.command_palette.is_open {
            let providers: Vec<Box<dyn crate::state::command_palette::CommandPaletteProvider>> = vec![
                Box::new(crate::state::command_palette_providers::AppCommandProvider),
                Box::new(crate::state::command_palette_providers::WorkspaceFileProvider),
                Box::new(crate::state::command_palette_providers::MarkdownContentProvider),
            ];
            crate::views::modals::command_palette::CommandPaletteModal::new(
                &mut self.state.command_palette,
                self.state.workspace.data.as_ref(),
                &mut self.pending_action,
                &providers,
            )
            .show(ctx);
        }

        if self.state.layout.show_search_modal {
            let mut is_open = true;
            let workspace_data = self.state.workspace.data.as_ref();
            crate::views::modals::search::SearchModal::new(
                &mut self.state.search,
                workspace_data,
                &mut is_open,
                &mut self.pending_action,
            )
            .show(ctx);
            if !is_open {
                self.pending_action = AppAction::ToggleSearchModal;
            }

            if self.state.search.md_history.recent_terms
                != self
                    .state
                    .config
                    .settings
                    .settings()
                    .search
                    .recent_md_queries
            {
                self.state
                    .config
                    .settings
                    .settings_mut()
                    .search
                    .recent_md_queries = self.state.search.md_history.recent_terms.clone();
                if !self.state.config.try_save_settings() {
                    tracing::warn!("Failed to save search history settings");
                }
            }
        }

        if self.show_about {
            crate::views::modals::about::AboutModal::new(
                &mut self.show_about,
                self.about_icon.as_ref(),
                &mut self.pending_action,
            )
            .show(ctx);
            if matches!(self.pending_action, AppAction::ShowReleaseNotes) {
                self.show_about = false;
            }
        }

        if let Some(path) = self.show_meta_info_for.clone() {
            let mut is_open = true;
            crate::views::modals::meta_info::MetaInfoModal::new(&mut is_open, &path).show(ctx);
            if !is_open {
                self.show_meta_info_for = None;
            }
        }

        if let Some(mut modal_data) = self.state.layout.create_fs_node_modal.take() {
            let visible_extensions = &self
                .state
                .config
                .settings
                .settings()
                .workspace
                .visible_extensions;
            let close = crate::views::modals::file_ops::CreateFsNodeModal::new(
                &mut modal_data,
                visible_extensions,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.create_fs_node_modal = Some(modal_data);
            }
        }
        if let Some(mut modal_data) = self.state.layout.rename_modal.take() {
            let close = crate::views::modals::file_ops::RenameModal::new(
                &mut modal_data,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.rename_modal = Some(modal_data);
            }
        }
        if let Some(modal_data) = self.state.layout.delete_modal.take() {
            let close = crate::views::modals::file_ops::DeleteModal::new(
                &modal_data,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.delete_modal = Some(modal_data);
            }
        }

        if self.show_update_dialog {
            crate::views::modals::update::UpdateModal::new(
                &mut self.show_update_dialog,
                &self.state,
                &mut self.update_markdown_cache,
                &mut self.pending_action,
            )
            .show(ctx);
        }

        self.state.scroll.scroll_to_line = None;
        crate::views::app_frame::intercept_url_commands(ctx, self);

        if let Some(start) = self.splash_start {
            let elapsed = start.elapsed().as_secs_f32();
            let dismissed =
                crate::views::splash::SplashOverlay::new(elapsed, self.about_icon.as_ref())
                    .show(ctx);
            if dismissed {
                self.splash_start = None;
            }
        }
    }

    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // WHY: eframe::App trait requires this method in some contexts or versions.
    }

    fn on_exit(&mut self) {
        self.save_workspace_state();
    }
}

#[cfg(test)]
include!("shell_ui_tests.rs");
