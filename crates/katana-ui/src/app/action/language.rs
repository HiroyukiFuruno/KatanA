use crate::shell::KatanaApp;

impl KatanaApp {
    pub(super) fn handle_action_change_language(&mut self, lang: String) {
        self.state.config.settings.settings_mut().language = lang;
        let effective_language = self
            .state
            .config
            .settings
            .resolve_effective_language(katana_platform::OsLocaleOps::get_default_language);
        crate::i18n::I18nOps::set_language(&effective_language);
        let runtime_language = crate::i18n::I18nOps::get_language();
        crate::shell_ui::ShellUiOps::update_native_menu_strings_from_i18n();
        if !self.state.config.try_save_settings() {
            tracing::warn!("Failed to save language setting");
        }
        /* WHY: Synchronize demo content localization if the demo is open */
        self.handle_action_switch_demo_lang(&runtime_language);
    }
}
