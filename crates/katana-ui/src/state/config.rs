use katana_core::plugin::PluginRegistry;
use katana_platform::{CacheFacade, SettingsService};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Behavior,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum SettingsTab {
    #[default]
    Theme,
    Font,
    Layout,
    Workspace,
    Updates,
    Behavior,
}

impl SettingsTab {
    pub const fn section(&self) -> SettingsSection {
        match self {
            Self::Theme | Self::Font | Self::Layout => SettingsSection::Appearance,
            Self::Workspace | Self::Updates | Self::Behavior => SettingsSection::Behavior,
        }
    }
}

impl SettingsSection {
    pub const fn tabs(&self) -> &[SettingsTab] {
        match self {
            Self::Appearance => &[SettingsTab::Theme, SettingsTab::Font, SettingsTab::Layout],
            Self::Behavior => &[
                SettingsTab::Workspace,
                SettingsTab::Updates,
                SettingsTab::Behavior,
            ],
        }
    }
}

pub struct ConfigState {
    pub plugin_registry: PluginRegistry,
    pub settings: SettingsService,
    pub cache: Arc<dyn CacheFacade>,
    pub active_settings_tab: SettingsTab,
    pub active_settings_section: SettingsSection,
    pub settings_tree_force_open: Option<bool>,
    pub settings_save_error: Option<String>,
}

impl ConfigState {
    pub fn new(
        plugin_registry: PluginRegistry,
        settings: SettingsService,
        cache: Arc<dyn CacheFacade>,
    ) -> Self {
        Self {
            plugin_registry,
            settings,
            cache,
            active_settings_tab: SettingsTab::default(),
            active_settings_section: SettingsSection::default(),
            settings_tree_force_open: None,
            settings_save_error: None,
        }
    }

    pub fn try_save_settings(&mut self) -> bool {
        match self.settings.save() {
            Ok(_) => {
                self.settings_save_error = None;
                true
            }
            Err(e) => {
                tracing::error!("Failed to save settings: {}", e);
                self.settings_save_error = Some(e.to_string());
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_platform::settings::repository::JsonFileRepository;
    use katana_platform::SettingsService;
    use tempfile::tempdir;

    struct DummyCache;
    impl CacheFacade for DummyCache {
        fn get_memory(&self, _key: &str) -> Option<String> {
            None
        }
        fn set_memory(&self, _key: &str, _value: String) {}
        fn get_persistent(&self, _key: &str) -> Option<String> {
            None
        }
        fn set_persistent(&self, _key: &str, _value: String) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_try_save_settings_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("settings.json");
        let repo = Box::new(JsonFileRepository::new(file_path.clone()));
        let settings = SettingsService::new(repo);

        let mut config = ConfigState::new(PluginRegistry::new(), settings, Arc::new(DummyCache));

        config.settings_save_error = Some("old error".into());
        let success = config.try_save_settings();
        assert!(success, "try_save_settings should succeed on valid path");
        assert!(
            config.settings_save_error.is_none(),
            "error should be cleared entirely on success"
        );
        assert!(file_path.exists(), "settings.json should be created");
    }

    #[test]
    fn test_try_save_settings_failure_is_caught() {
        // Give it a directory path instead of a file, forcing write failure
        let dir = tempdir().unwrap();
        let repo = Box::new(JsonFileRepository::new(dir.path().to_path_buf()));
        let settings = SettingsService::new(repo);

        let mut config = ConfigState::new(PluginRegistry::new(), settings, Arc::new(DummyCache));

        let success = config.try_save_settings();
        assert!(
            !success,
            "try_save_settings should fail gracefully when atomic write fails"
        );
        assert!(
            config.settings_save_error.is_some(),
            "settings_save_error should be populated with the failure message"
        );
    }
}
