/* WHY: Settings persistence layer.

Provides the `SettingsRepository` trait with JSON file and in-memory implementations. */

use std::path::PathBuf;

use super::migration::MigrationRunner;
use super::migration::{v0_1_2, v0_1_3_to_0_1_4, v0_1_4_to_0_2_0};
use super::types::{AppSettings, SettingsLoadOrigin};

// WHY: Minimal interface for loading and saving settings.
pub trait SettingsRepository: Send {
    fn load(&self) -> AppSettings;
    #[allow(clippy::missing_errors_doc)]
    fn save(&self, settings: &AppSettings) -> anyhow::Result<()>;
    // WHY: Returns the load origin for detecting first launch.
    fn load_origin(&self) -> SettingsLoadOrigin {
        // WHY: Default: assume persisted to avoid false positives in tests.
        SettingsLoadOrigin::Persisted
    }

    // WHY: Load structured workspace state (e.g. tabs, pins) distinct from transient cache.
    fn load_workspace_state(&self, workspace_key: &str) -> Option<String>;

    // WHY: Save structured workspace state.
    #[allow(clippy::missing_errors_doc)]
    fn save_workspace_state(&self, workspace_key: &str, state_json: &str) -> anyhow::Result<()>;
}

// WHY: ── JSON file repository ──

// WHY: Persists settings as a JSON file on disk.
pub struct JsonFileRepository {
    pub(crate) path: PathBuf,
}

impl JsonFileRepository {
    // WHY: Create a repository targeting the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /* WHY: Create a repository using the platform-standard config directory.
    On macOS: `~/Library/Application Support/KatanA/settings.json` */
    pub fn with_default_path() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("KatanA").join("settings.json"))
    }

    fn backup_corrupt_file(&self, reason: &str) {
        let backup_path = self.path.with_extension("bak");
        tracing::error!(
            "Settings file corrupted ({}). Backing up to: {}",
            reason,
            backup_path.display()
        );
        if self.path.exists()
            && let Err(e) = std::fs::rename(&self.path, &backup_path)
        {
            tracing::error!("Failed to create settings backup: {}", e);
        }
    }
}

impl SettingsRepository for JsonFileRepository {
    fn load(&self) -> AppSettings {
        match std::fs::read_to_string(&self.path) {
            Ok(json_str) => {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
                match parsed {
                    Ok(mut value) => {
                        let mut runner = MigrationRunner::new();
                        runner.add_strategy(Box::new(v0_1_2::Migration0_1_2));
                        runner.add_strategy(Box::new(v0_1_3_to_0_1_4::Migration013To014));
                        runner.add_strategy(Box::new(v0_1_4_to_0_2_0::Migration014To020));
                        value = runner.migrate(value);

                        match serde_json::from_value(value) {
                            Ok(settings) => settings,
                            Err(e) => {
                                self.backup_corrupt_file(&e.to_string());
                                AppSettings::default()
                            }
                        }
                    }
                    Err(e) => {
                        self.backup_corrupt_file(&e.to_string());
                        AppSettings::default()
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
            Err(e) => {
                tracing::error!("Failed to read settings file: {}", e);
                AppSettings::default()
            }
        }
    }

    fn save(&self, settings: &AppSettings) -> anyhow::Result<()> {
        let parent = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(settings)?;

        let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
        use std::io::Write;
        temp_file.write_all(json.as_bytes())?;
        temp_file.flush()?; // WHY: Ensure data is fully flushed to OS before persisting

        temp_file.persist(&self.path)?;

        tracing::info!("Settings saved to {}", self.path.display());
        Ok(())
    }

    fn load_origin(&self) -> SettingsLoadOrigin {
        if self.path.exists() {
            SettingsLoadOrigin::Persisted
        } else {
            SettingsLoadOrigin::FirstLaunch
        }
    }

    fn load_workspace_state(&self, workspace_key: &str) -> Option<String> {
        let parent = self.path.parent()?;
        let path = parent
            .join("workspaces")
            .join(format!("{}.json", workspace_key));
        std::fs::read_to_string(path).ok()
    }

    fn save_workspace_state(&self, workspace_key: &str, state_json: &str) -> anyhow::Result<()> {
        let parent = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let workspaces_dir = parent.join("workspaces");
        std::fs::create_dir_all(&workspaces_dir)?;

        let path = workspaces_dir.join(format!("{}.json", workspace_key));
        let mut temp_file = tempfile::NamedTempFile::new_in(&workspaces_dir)?;
        use std::io::Write;
        temp_file.write_all(state_json.as_bytes())?;
        temp_file.flush()?;
        temp_file.persist(&path)?;
        Ok(())
    }
}

// WHY: ── In-memory repository (for tests) ──

// WHY: No-op repository that never touches the filesystem.
pub struct InMemoryRepository;

impl SettingsRepository for InMemoryRepository {
    fn load(&self) -> AppSettings {
        AppSettings::default()
    }

    fn save(&self, _settings: &AppSettings) -> anyhow::Result<()> {
        Ok(())
    }

    fn load_workspace_state(&self, _workspace_key: &str) -> Option<String> {
        None
    }

    fn save_workspace_state(&self, _workspace_key: &str, _state_json: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
