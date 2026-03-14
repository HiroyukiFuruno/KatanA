//! Application settings persistence (MVP: in-memory only).
//!
//! The settings service owns all persistent configuration. In the MVP the
//! settings are held in memory and not written to disk; the persistence
//! backend can be layered in later without changing the public API.

use std::collections::HashMap;

/// Application-level settings.
#[derive(Debug, Clone, Default)]
pub struct AppSettings {
    /// ID of the last opened workspace root path, restored on next launch.
    pub last_workspace: Option<String>,
    /// Active AI provider identifier.
    pub ai_provider: Option<String>,
    /// Additional key-value settings for future use.
    pub extra: HashMap<String, String>,
}

/// Platform settings service.
pub struct SettingsService {
    settings: AppSettings,
}

impl SettingsService {
    pub fn new() -> Self {
        Self {
            settings: AppSettings::default(),
        }
    }

    /// Load settings from the given path (MVP: ignores path, returns defaults).
    pub fn load_from(_path: &str) -> Self {
        // Future: deserialize from TOML/JSON at `path`.
        Self::new()
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}
