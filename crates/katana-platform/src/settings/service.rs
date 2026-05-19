/* WHY: Application settings service.

`SettingsService` handles reading and writing settings via the repository,
and manages OS integration (theme, language) on first launch. */

use super::defaults::{AUTO_LANGUAGE_CODE, SettingsDefaultOps};
use super::repository::{InMemoryRepository, SettingsRepository};
use super::types::{AppSettings, SettingsLoadOrigin};

/* WHY: Platform settings service. */
pub struct SettingsService {
    settings: AppSettings,
    repository: Box<dyn SettingsRepository>,
    /* WHY: `true` when the settings were first loaded without an existing settings file. */
    is_first_launch: bool,
}

impl SettingsService {
    /* WHY: Create a new service backed by the given repository, loading initial settings. */
    pub fn new(repository: Box<dyn SettingsRepository>) -> Self {
        let is_first_launch = repository.load_origin() == SettingsLoadOrigin::FirstLaunch;
        let settings = repository.load();
        Self {
            settings,
            repository,
            is_first_launch,
        }
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /* WHY: Persist current settings via the repository. */
    #[allow(clippy::missing_errors_doc)]
    pub fn save(&self) -> anyhow::Result<()> {
        self.repository.save(&self.settings)
    }

    /* WHY: Applies the OS-default theme preset on first launch only.
    If this is not a first launch (settings file already existed), this is a no-op
    to respect the user's saved theme preference. */
    pub fn apply_os_default_theme(&mut self) {
        if !self.is_first_launch {
            /* WHY: Existing users keep their saved preset unchanged. */
            return;
        }
        let preset = SettingsDefaultOps::select_initial_preset();
        self.settings.theme.preset = preset;
        self.settings.theme.theme = preset.colors().mode.to_theme_string();
    }

    /* WHY: Stores auto-follow language mode on first launch. */
    pub fn apply_os_default_language(&mut self) {
        if !self.is_first_launch {
            return;
        }
        self.settings.language = SettingsDefaultOps::default_language();
    }

    pub fn resolve_effective_language(&self, detector: impl FnOnce() -> Option<String>) -> String {
        let saved = self.settings.language.trim();
        if !Self::is_auto_language(saved) {
            return saved.to_string();
        }

        detector()
            .map(|locale| crate::OsLocaleOps::resolve_locale_to_lang(&locale))
            .unwrap_or_else(SettingsDefaultOps::fallback_language)
    }

    fn is_auto_language(language: &str) -> bool {
        language.is_empty() || language == AUTO_LANGUAGE_CODE
    }

    /* WHY: Load structured workspace state (e.g. tabs, pins) from the config directory. */
    pub fn load_workspace_state(&self, workspace_key: &str) -> Option<String> {
        self.repository.load_workspace_state(workspace_key)
    }

    /* WHY: Save structured workspace state to the config directory. */
    #[allow(clippy::missing_errors_doc)]
    pub fn save_workspace_state(
        &self,
        workspace_key: &str,
        state_json: &str,
    ) -> anyhow::Result<()> {
        self.repository
            .save_workspace_state(workspace_key, state_json)
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new(Box::new(InMemoryRepository))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepository {
        is_first_launch: bool,
    }
    impl SettingsRepository for MockRepository {
        fn load(&self) -> AppSettings {
            AppSettings::default()
        }
        fn save(&self, _settings: &AppSettings) -> anyhow::Result<()> {
            Ok(())
        }
        fn load_origin(&self) -> SettingsLoadOrigin {
            if self.is_first_launch {
                SettingsLoadOrigin::FirstLaunch
            } else {
                SettingsLoadOrigin::Persisted
            }
        }
        fn load_workspace_state(&self, _workspace_key: &str) -> Option<String> {
            None
        }
        fn save_workspace_state(
            &self,
            _workspace_key: &str,
            _state_json: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_apply_os_default_language_first_launch() {
        let repo = Box::new(MockRepository {
            is_first_launch: true,
        });
        let mut service = SettingsService::new(repo);
        assert!(service.is_first_launch);

        service.apply_os_default_language();
        assert_eq!(
            service.settings().language,
            SettingsDefaultOps::default_language()
        );
    }

    #[test]
    fn test_apply_os_default_language_existing_user() {
        let repo = Box::new(MockRepository {
            is_first_launch: false,
        });
        /* WHY: Simulate existing user */
        let mut service = SettingsService::new(repo);
        service.settings_mut().language = "ja".to_string();

        assert!(!service.is_first_launch);

        service.apply_os_default_language();
        /* WHY: Existing user shouldn't be overridden */
        assert_eq!(service.settings().language, "ja");
    }
}
