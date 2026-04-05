use super::types::*;
use crate::shell::KatanaApp;
use crate::shell_logic::ShellLogicOps;
use crate::theme_bridge;
use eframe::egui;

impl<'a> WindowTitle<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        let ws_root_for_title = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());

        let title_text = match app.state.active_document() {
            Some(doc) => {
                let fname = doc.file_name().unwrap_or("");
                let rel =
                    ShellLogicOps::relative_full_path(&doc.path, ws_root_for_title.as_deref());
                crate::shell_logic::ShellLogicOps::format_window_title(
                    fname,
                    &rel,
                    &crate::i18n::I18nOps::get().menu.release_notes,
                )
            }
            None => "KatanA".to_string(),
        };

        if app.state.layout.last_window_title != title_text {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));
            app.state.layout.last_window_title = title_text;
        }
    }
}

impl<'a> TitleBar<'a> {
    pub(crate) fn new(
        app: &'a KatanaApp,
        theme_colors: &'a katana_platform::theme::ThemeColors,
    ) -> Self {
        Self { app, theme_colors }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        let theme_colors = self.theme_colors;
        let title_text = &app.state.layout.last_window_title;

        egui::Panel::top("app_title_bar").show_inside(ui, |ui| {
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.centered_and_justified(|ui| {
                        let title_color = theme_bridge::ThemeBridgeOps::rgb_to_color32(
                            theme_colors.system.title_bar_text,
                        );
                        ui.label(egui::RichText::new(title_text).small().color(title_color));
                    });
                })
                .show(ui);
        });
    }
}
