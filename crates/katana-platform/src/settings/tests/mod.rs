/* WHY: Root module for settings tests. Organizes tests by category into sub-modules for better maintainability and to comply with file length limits. */

use super::*;
use crate::theme::{ThemeColors, ThemePreset};
use defaults::SettingsDefaultOps;
use tempfile::TempDir;

mod defaults_test;
mod repositories;
mod serde_roundtrip;
mod service;
mod theme_colors;

/* WHY: Helper: a test repository that reports `FirstLaunch` and holds a preset. */
struct FirstLaunchRepo {
    preset: ThemePreset,
}

impl SettingsRepository for FirstLaunchRepo {
    fn load_workspace_state(&self, _workspace_key: &str) -> Option<String> {
        None
    }

    fn save_workspace_state(&self, _workspace_key: &str, _state_json: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn load(&self) -> AppSettings {
        let mut s = AppSettings::default();
        s.theme.preset = self.preset;
        s
    }

    fn save(&self, _settings: &AppSettings) -> anyhow::Result<()> {
        Ok(())
    }

    fn load_origin(&self) -> SettingsLoadOrigin {
        SettingsLoadOrigin::FirstLaunch
    }
}
