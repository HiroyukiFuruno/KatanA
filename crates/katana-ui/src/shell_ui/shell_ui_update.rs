use crate::app::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_ui::ShellUiOps;
use eframe::egui;

impl KatanaApp {
    /// Per-frame state updates: auto-save, auto-refresh, theme, font, shortcuts, polling.
    /// Returns resolved theme colors for the rendering phase.
    pub(super) fn poll_and_prepare(
        &mut self,
        ctx: &egui::Context,
    ) -> katana_platform::theme::ThemeColors {
        if self.needs_splash {
            self.splash_start = Some(std::time::Instant::now());
            self.needs_splash = false;
        }

        self.tick_auto_save();
        self.tick_auto_refresh(ctx);

        let theme_colors = self
            .state
            .config
            .settings
            .settings()
            .effective_theme_colors();
        if self.cached_theme.as_ref() != Some(&theme_colors) {
            self.apply_theme(ctx, &theme_colors);
        }

        let font_size = self.state.config.settings.settings().clamped_font_size();
        if self.cached_font_size != Some(font_size) {
            crate::theme_bridge::ThemeBridgeOps::apply_font_size(ctx, font_size);
            self.cached_font_size = Some(font_size);
        }

        let font_family = self.state.config.settings.settings().font.family.clone();
        if self.cached_font_family.as_deref() != Some(&font_family) {
            crate::theme_bridge::ThemeBridgeOps::apply_font_family(ctx, &font_family);
            self.cached_font_family = Some(font_family);
        }

        self.handle_shortcuts(ctx);
        self.poll_download(ctx);
        self.poll_explorer_load(ctx);

        if let Some(path) = self.pending_document_loads.pop_front() {
            self.handle_select_document(path, false);
            ctx.request_repaint();
        }

        self.poll_update_install(ctx);
        self.poll_update_check(ctx);
        self.poll_changelog(ctx);
        self.poll_export(ctx);

        let native_action = crate::native_menu::NativeMenuOps::poll(
            &mut self.show_about,
            ShellUiOps::open_folder_dialog,
        );
        if !matches!(native_action, AppAction::None) {
            self.pending_action = native_action;
        }

        let action = self.take_action();
        crate::views::panels::preview::PreviewLogicOps::invalidate_preview_image_cache(
            ctx, &action,
        );
        self.process_action(ctx, action);

        theme_colors
    }

    fn tick_auto_save(&mut self) {
        let behavior = self.state.config.settings.settings().behavior.clone();
        if behavior.auto_save && behavior.auto_save_interval_secs > 0.0 {
            let now = std::time::Instant::now();
            match self.state.document.last_auto_save {
                Some(last)
                    if now.duration_since(last).as_secs_f64()
                        >= behavior.auto_save_interval_secs =>
                {
                    if self.state.active_document().is_some_and(|d| d.is_dirty) {
                        self.pending_action = crate::app_state::AppAction::SaveDocument;
                    }
                    self.state.document.last_auto_save = Some(now);
                }
                None => self.state.document.last_auto_save = Some(now),
                _ => {}
            }
        } else {
            self.state.document.last_auto_save = None;
        }
    }

    fn tick_auto_refresh(&mut self, ctx: &egui::Context) {
        let behavior = self.state.config.settings.settings().behavior.clone();
        if behavior.auto_refresh && behavior.auto_refresh_interval_secs > 0.0 {
            let now = std::time::Instant::now();
            match self.state.document.last_auto_refresh {
                Some(last)
                    if now.duration_since(last).as_secs_f64()
                        >= behavior.auto_refresh_interval_secs =>
                {
                    if self.state.active_document().is_some() {
                        self.pending_action =
                            crate::app_state::AppAction::RefreshDocument { is_manual: false };
                    }
                    self.state.document.last_auto_refresh = Some(now);
                }
                None => self.state.document.last_auto_refresh = Some(now),
                _ => {}
            }
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(
                behavior.auto_refresh_interval_secs,
            ));
        } else {
            self.state.document.last_auto_refresh = None;
        }
    }

    pub(super) fn apply_theme(
        &mut self,
        ctx: &egui::Context,
        theme_colors: &katana_platform::theme::ThemeColors,
    ) {
        let dark = theme_colors.mode == katana_platform::theme::ThemeMode::Dark;
        ctx.set_visuals(crate::theme_bridge::ThemeBridgeOps::visuals_from_theme(
            theme_colors,
        ));
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("katana_theme_colors"), theme_colors.clone()));
        ctx.global_style_mut(|s| s.spacing.scroll.floating = false);
        katana_core::markdown::color_preset::DiagramColorPreset::set_dark_mode(dark);
        self.cached_theme = Some(theme_colors.clone());
        if matches!(self.pending_action, AppAction::None) {
            self.pending_action = AppAction::RefreshDiagrams;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let cmd_shift_t = egui::KeyboardShortcut::new(
            egui::Modifiers {
                command: true,
                shift: true,
                ..Default::default()
            },
            egui::Key::T,
        );
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_shift_t))
            && !self.state.document.recently_closed_tabs.is_empty()
        {
            self.pending_action = AppAction::RestoreClosedDocument;
        }

        let cmd_p = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::P);
        let cmd_shift_p = egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::P,
        );
        let cmd_k = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::K);
        if ctx.input_mut(|i| {
            i.consume_shortcut(&cmd_p)
                || i.consume_shortcut(&cmd_shift_p)
                || i.consume_shortcut(&cmd_k)
        }) {
            self.pending_action = AppAction::ToggleCommandPalette;
        }

        let cmd_f = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::F);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_f)) {
            if !self.state.search.doc_search_open {
                self.state.search.doc_search_open = true;
                ctx.memory_mut(|m| {
                    m.data
                        .insert_temp(egui::Id::new("search_newly_opened"), true)
                });
                self.trigger_action(AppAction::DocSearchQueryChanged);
            } else {
                self.state.search.doc_search_open = false;
                self.state.search.doc_search_matches.clear();
            }
        }
    }
}
