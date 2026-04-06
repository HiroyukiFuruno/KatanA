use crate::app::*;
use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(super) fn show_main_panels(
        &mut self,
        ctx: &egui::Context,
        theme_colors: &katana_platform::theme::ThemeColors,
    ) -> bool {
        let accepted_ver = self
            .state
            .config
            .settings
            .settings()
            .terms_accepted_version
            .as_ref();
        if accepted_ver != Some(&crate::about_info::APP_VERSION.to_string()) {
            egui::CentralPanel::default().show(ctx, |ui| {
                crate::views::modals::terms::TermsModal::new(
                    crate::about_info::APP_VERSION,
                    &mut self.pending_action,
                )
                .show(ui);
            });
            return false;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                let download_req =
                    crate::views::app_frame::MainPanels::new(self, theme_colors).show(ui);
                if let Some(req) = download_req {
                    self.start_download(req);
                }
            });
        true
    }

    pub(super) fn show_modals(&mut self, ctx: &egui::Context) {
        if self.state.layout.show_slideshow {
            if let Some(doc) = self.state.active_document() {
                if let Some(preview) = self.tab_previews.iter_mut().find(|p| p.path == doc.path) {
                    crate::preview_pane::FullscreenLogicOps::render_slideshow_modal(
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
                .filter(|_| matches!(self.pending_action, AppAction::None))
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
            crate::views::modals::search::SearchModal::new(
                &mut self.state.search,
                self.state.workspace.data.as_ref(),
                &mut is_open,
                &mut self.pending_action,
            )
            .show(ctx);
            if !is_open && matches!(self.pending_action, AppAction::None) {
                self.pending_action = AppAction::ToggleSearchModal;
            }

            let recent = self.state.search.md_history.recent_terms.clone();
            let saved = self
                .state
                .config
                .settings
                .settings()
                .search
                .recent_md_queries
                .clone();
            if recent != saved {
                self.state
                    .config
                    .settings
                    .settings_mut()
                    .search
                    .recent_md_queries = recent;
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
            let visible_ext = self
                .state
                .config
                .settings
                .settings()
                .workspace
                .visible_extensions
                .clone();
            let close = crate::views::modals::file_ops::CreateFsNodeModal::new(
                &mut modal_data,
                &visible_ext,
                &mut self.pending_action,
            )
            .show(ctx);
            if !close {
                self.state.layout.create_fs_node_modal = Some(modal_data);
            }
        }
        if let Some(mut modal_data) = self.state.layout.rename_modal.take()
            && !crate::views::modals::file_ops::RenameModal::new(
                &mut modal_data,
                &mut self.pending_action,
            )
            .show(ctx)
        {
            self.state.layout.rename_modal = Some(modal_data);
        }
        if let Some(modal_data) = self.state.layout.delete_modal.take()
            && !crate::views::modals::file_ops::DeleteModal::new(
                &modal_data,
                &mut self.pending_action,
            )
            .show(ctx)
        {
            self.state.layout.delete_modal = Some(modal_data);
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
    }

    pub(super) fn show_splash(&mut self, ctx: &egui::Context) {
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
}
