use super::PreviewSidePanels;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct SidebarButton<'a> {
    pub(super) available: bool,
    pub(super) icon: crate::Icon,
    pub(super) active: bool,
    pub(super) label: &'a str,
    pub(super) shortcut: Option<&'a str>,
    pub(super) action: AppAction,
}
impl PreviewSidePanels<'_> {
    pub(super) fn render_info_control(&mut self, ui: &mut egui::Ui, available: bool) {
        let response = ui
            .add_enabled_ui(available, |ui| {
                self.render_toggle_button(
                    ui,
                    crate::Icon::Info,
                    false,
                    &crate::i18n::I18nOps::get().meta_info.title,
                    None,
                )
            })
            .inner;
        if response.clicked()
            && let Some(document) = self.app.state.active_document()
        {
            self.app.pending_action = AppAction::ShowMetaInfo(document.path.clone());
        }
    }

    pub(super) fn render_action_button(
        &mut self,
        ui: &mut egui::Ui,
        button: SidebarButton<'_>,
    ) -> egui::Response {
        let response = ui
            .add_enabled_ui(button.available, |ui| {
                self.render_toggle_button(
                    ui,
                    button.icon,
                    button.active,
                    button.label,
                    button.shortcut,
                )
            })
            .inner;
        if response.clicked() {
            self.app.pending_action = button.action;
        }
        response
    }
}
