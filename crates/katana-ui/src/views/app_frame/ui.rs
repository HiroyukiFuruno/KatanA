use super::types::*;
use crate::app::action::ActionOps;
use crate::app_state::AppAction;
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use eframe::egui;

impl<'a> MainPanels<'a> {
    pub fn new(
        app: &'a mut KatanaApp,
        theme_colors: &'a katana_platform::theme::ThemeColors,
    ) -> Self {
        Self { app, theme_colors }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let app = self.app;

        let theme_colors = self.theme_colors;
        let export_filenames: Vec<String> = app
            .export_tasks
            .iter()
            .map(|t| t.filename.clone())
            .collect();

        let mut resolved_status_ref = app.state.layout.status_message.as_ref();
        let settings_err_tuple;
        if let Some(err) = &app.state.config.settings_save_error {
            settings_err_tuple = (err.clone(), crate::app_state::StatusType::Error);
            resolved_status_ref = Some(&settings_err_tuple);
        }

        let is_dirty = app.state.is_dirty();
        let total_problems = app.state.diagnostics.total_problems();

        let action = Self::render_status_bar(
            ui,
            resolved_status_ref,
            &export_filenames,
            is_dirty,
            total_problems,
        );
        if let Some(a) = action {
            app.pending_action = a;
        }

        crate::views::panels::problems::ProblemsPanel::new(&mut app.state, &mut app.pending_action)
            .show(ui);

        WindowTitle::new(app).show(ui);
        GlobalMenuBar::new(app).show(ui);
        TitleBar::new(app, theme_colors).show(ui);
        ExplorerSidebar::new(app).show(ui);
        crate::views::panels::chat::ChatPanel::new(app).show(ui);
        TabToolbar::new(app).show(ui);
        CentralContent::new(app).show(ui)
    }

    fn render_status_bar(
        ui: &mut egui::Ui,
        resolved_status: Option<&(String, crate::app_state::StatusType)>,
        export_filenames: &[String],
        is_dirty: bool,
        total_problems: usize,
    ) -> Option<crate::app_state::AppAction> {
        let mut out_action = None;
        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            let action =
                crate::views::top_bar::StatusBar::new(resolved_status, is_dirty, export_filenames)
                    .show(ui, total_problems);

            if let Some(a) = action {
                out_action = Some(a);
            }
        });
        out_action
    }
}

impl AppFrameOps {
    pub(crate) fn intercept_url_commands(ctx: &egui::Context, app: &mut KatanaApp) {
        let commands = ctx.output_mut(|o| std::mem::take(&mut o.commands));
        let mut unprocessed_commands = Vec::new();

        for cmd in commands {
            let open = match &cmd {
                egui::OutputCommand::OpenUrl(o) => o,
                _ => {
                    unprocessed_commands.push(cmd);
                    continue;
                }
            };

            let url = &open.url;
            if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("mailto:")
            {
                unprocessed_commands.push(cmd);
                continue;
            }

            if url.starts_with("Katana://Command/SwitchDemoLanguage?lang=") {
                let lang = url
                    .strip_prefix("Katana://Command/SwitchDemoLanguage?lang=")
                    .unwrap();
                app.process_action(ctx, AppAction::SwitchDemoLanguage(lang.to_string()));
                continue;
            }

            if url.starts_with("Katana://") {
                app.process_action(
                    ctx,
                    AppAction::SelectDocument(std::path::PathBuf::from(url)),
                );
                continue;
            }

            let mut path = std::path::PathBuf::from(url);
            if path.is_relative()
                && let Some(parent) = app
                    .state
                    .active_document()
                    .and_then(|doc| doc.path.parent())
            {
                path = parent.join(path);
            }
            app.process_action(ctx, AppAction::SelectDocument(path));
        }
        ctx.output_mut(|o| o.commands.extend(unprocessed_commands));
    }
}
