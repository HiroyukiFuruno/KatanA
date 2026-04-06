use super::migration::MigrationRunner;
use super::migration::{v0_1_2, v0_1_3_to_0_1_4, v0_1_4_to_0_2_0, v0_2_0_to_0_2_1};
use super::types::{AppSettings, SettingsLoadOrigin};
use std::path::PathBuf;

pub trait SettingsRepository: Send {
    fn load(&self) -> AppSettings;
    /// # Errors
    /// Returns an error if the settings cannot be serialized or written to disk.
    fn save(&self, settings: &AppSettings) -> anyhow::Result<()>;
    fn load_origin(&self) -> SettingsLoadOrigin {
        SettingsLoadOrigin::Persisted
    }
    fn load_workspace_state(&self, workspace_key: &str) -> Option<String>;
    /// # Errors
    /// Returns an error if the workspace state cannot be serialized or written to the respective file.
    fn save_workspace_state(&self, workspace_key: &str, state_json: &str) -> anyhow::Result<()>;
}

pub struct JsonFileRepository {
    pub(crate) path: PathBuf,
}

impl JsonFileRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn with_default_path() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("KatanA").join("settings.json"))
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
                        runner.add_strategy(Box::new(v0_2_0_to_0_2_1::Migration020To021));
                        value = runner.migrate(value);
                        match serde_json::from_value::<AppSettings>(value) {
                            Ok(mut settings) => {
                                /* WHY: Add any missing items to the rail order */
                                settings.layout.normalize();
                                settings
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to deserialize settings: {}. Path: {:?}",
                                    e,
                                    self.path
                                );
                                AppSettings::default()
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to parse settings JSON: {}. Path: {:?}",
                            e,
                            self.path
                        );
                        AppSettings::default()
                    }
                }
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                tracing::error!("Failed to read settings file: {}. Path: {:?}", e, self.path);
                AppSettings::default()
            }
            Err(_) => AppSettings::default(),
        }
    }
    fn save(&self, settings: &AppSettings) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(settings)?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, json)?;
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
            .ok_or_else(|| anyhow::anyhow!("No parent dir"))?;
        let workspaces_dir = parent.join("workspaces");
        std::fs::create_dir_all(&workspaces_dir)?;
        let path = workspaces_dir.join(format!("{}.json", workspace_key));
        std::fs::write(path, state_json)?;
        Ok(())
    }
}

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
