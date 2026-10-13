use crate::app::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use crate::shell_ui::ShellUiOps;
use eframe::egui;
use katana_platform::theme::Rgb;

const TABLE_HEADER_ALPHA: f32 = 0.3;
const TABLE_STRIPE_ALPHA: f32 = 0.1;

impl KatanaApp {
    /// Per-frame state updates and rendering prerequisites.
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
        self.sync_html_preview_observer();
        self.tick_pending_html_preview_refresh();
        self.poll_html_preview_observer();

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
        super::dropped_files::DroppedFileOps::queue(self, ctx);
        self.poll_explorer_load(ctx);

        if let Some(path) = self.pending_document_loads.pop_front() {
            self.handle_select_document(path, false);
            ctx.request_repaint();
        }

        self.tick_update_check(ctx);
        self.poll_update_install(ctx);
        self.poll_update_check(ctx);
        self.poll_changelog(ctx);
        self.poll_export(ctx);
        self.poll_linter_docs(ctx);
        self.poll_url_source(ctx);
        self.poll_html_browser_navigation(ctx);
        self.tick_diagnostics(ctx);

        let editor_focused = matches!(
            crate::state::shortcut_context::ShortcutContextResolver::resolve(&self.state, ctx),
            crate::state::shortcut_context::ShortcutContext::Editor
        );
        crate::native_menu::NativeMenuOps::update_availability(&self.state, editor_focused);
        let native_action = crate::native_menu::NativeMenuOps::poll(ShellUiOps::open_folder_dialog);
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

    pub(crate) fn update_file_dialog(&mut self, ctx: &egui::Context) {
        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_picked()
            && let Some(action) = self.pending_dialog_action.take()
        {
            self.handle_file_dialog_action(path, action);
        }
    }

    fn handle_file_dialog_action(&mut self, path: std::path::PathBuf, action: AppAction) {
        match action {
            AppAction::PickOpenWorkspace => {
                self.handle_open_explorer(path);
            }
            AppAction::PickOpenFileInCurrentWorkspace => {
                crate::app::action::FileOpenOps::open_in_current_workspace(self, path);
            }
            AppAction::PickOpenFileInNewWorkspace => {
                crate::app::action::FileOpenOps::open_as_temporary_workspace(self, path);
            }
            AppAction::IngestImageFile => {
                let Ok(bytes) = std::fs::read(&path) else {
                    return;
                };
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png");
                self.process_image_ingest(&bytes, ext);
            }
            AppAction::PickExportDocument {
                doc_path,
                ext,
                source,
            } => {
                use crate::app::export_poll::ExportPoll;
                self.perform_tool_export(&source, &ext, path, &doc_path);
            }
            _ => {}
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
        katana_core::markdown::DiagramThemeSnapshot::set_current_override(
            katana_core::markdown::DiagramThemeOverride {
                name: theme_colors.name.clone(),
                is_dark: dark,
                background: rgb_hex(theme_colors.preview.background),
                text: rgb_hex(theme_colors.preview.text),
                preview_text: rgb_hex(theme_colors.preview.text),
                table_border: Some(rgb_hex(theme_colors.preview.border)),
                table_header_background: Some(blended_preview_selection_hex(
                    theme_colors.preview.selection,
                    theme_colors.preview.background,
                    TABLE_HEADER_ALPHA,
                )),
                table_even_row_background: Some(blended_preview_selection_hex(
                    theme_colors.preview.selection,
                    theme_colors.preview.background,
                    TABLE_STRIPE_ALPHA,
                )),
            },
        );
        self.cached_theme = Some(theme_colors.clone());
        if matches!(self.pending_action, AppAction::None) {
            self.pending_action = AppAction::RefreshDiagrams;
        }
    }

    fn tick_diagnostics(&mut self, ctx: &egui::Context) {
        const DIAGNOSTICS_DEBOUNCE_MS: u128 = 500;
        if let Some(last) = self.state.diagnostics.last_buffer_update {
            let elapsed = last.elapsed().as_millis();
            if elapsed > DIAGNOSTICS_DEBOUNCE_MS {
                self.state.diagnostics.last_buffer_update = None;
                if matches!(self.pending_action, crate::app_state::AppAction::None) {
                    self.pending_action = crate::app_state::AppAction::RefreshDiagnostics;
                }
            } else {
                let remaining = (DIAGNOSTICS_DEBOUNCE_MS - elapsed) as u64;
                ctx.request_repaint_after(std::time::Duration::from_millis(remaining));
            }
        }
    }
}

fn rgb_hex(color: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

fn blended_preview_selection_hex(selection: Rgb, background: Rgb, alpha: f32) -> String {
    let background = crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(background);
    let selection = crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(selection);
    let blended = background.blend(selection.gamma_multiply(alpha));
    format!("#{:02x}{:02x}{:02x}", blended.r(), blended.g(), blended.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blended_preview_selection_hex_matches_viewer_table_fill_over_background() {
        let selection = Rgb {
            r: 0,
            g: 120,
            b: 212,
        };
        let background = Rgb {
            r: 30,
            g: 30,
            b: 30,
        };

        let hex = blended_preview_selection_hex(selection, background, TABLE_HEADER_ALPHA);

        assert_eq!(hex, "#153955");
    }
}
