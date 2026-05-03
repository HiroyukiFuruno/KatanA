use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use eframe::egui;

pub(super) struct GlobalMenuContext<'a> {
    app: &'a mut KatanaApp,
    i18n: &'a crate::i18n::I18nMessages,
}

impl<'a> GlobalMenuContext<'a> {
    pub(super) fn new(app: &'a mut KatanaApp, i18n: &'a crate::i18n::I18nMessages) -> Self {
        Self { app, i18n }
    }

    pub(super) fn i18n(&self) -> &crate::i18n::I18nMessages {
        self.i18n
    }

    pub(super) fn action_item(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        label: &str,
        action: AppAction,
    ) {
        let button = egui::Button::new(label);
        self.apply_button(ui, id, button, action);
    }

    pub(super) fn shortcut_action_item(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        label: &str,
        shortcut: &str,
        action: AppAction,
    ) {
        let shortcut = crate::os_command::OsCommandOps::get(shortcut);
        let button = egui::Button::new(label).shortcut_text(shortcut);
        self.apply_button(ui, id, button, action);
    }

    pub(super) fn select_language(&mut self, ui: &mut egui::Ui, code: &str, label: &str) {
        let current = self.app.state.config.settings.settings().language.clone();
        if ui
            .radio(self.is_language_selected(&current, code), label)
            .clicked()
        {
            self.set_action(ui, AppAction::ChangeLanguage(code.to_string()));
        }
    }

    pub(super) fn set_action(&mut self, ui: &mut egui::Ui, action: AppAction) {
        self.app.pending_action = action;
        ui.close_menu();
    }

    fn apply_button(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        button: egui::Button<'_>,
        action: AppAction,
    ) {
        if ui.add_enabled(self.is_available(id), button).clicked() {
            self.set_action(ui, action);
        }
    }

    fn is_available(&self, id: &str) -> bool {
        crate::state::command_inventory::CommandInventory::all()
            .iter()
            .find(|command| command.id == id)
            .is_some_and(|command| (command.is_available)(self.state()))
    }

    fn state(&self) -> &AppState {
        &self.app.state
    }

    fn is_language_selected(&self, current: &str, code: &str) -> bool {
        let default = katana_platform::OsLocaleOps::get_default_language();
        current == code || (current.is_empty() && default.as_deref() == Some(code))
    }
}
