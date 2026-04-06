use super::types::GlobalWorkspaceState;
use std::path::PathBuf;

pub trait GlobalWorkspaceRepository: Send {
    fn load(&self) -> GlobalWorkspaceState;
    /// # Errors
    /// Returns an error if the workspace state cannot be serialized or written to disk.
    fn save(&self, state: &GlobalWorkspaceState) -> anyhow::Result<()>;
    fn is_ephemeral(&self) -> bool {
        false
    }
}

pub struct JsonWorkspaceRepository {
    pub(crate) path: PathBuf,
}

impl JsonWorkspaceRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn with_default_path() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("KatanA").join("workspace.json"))
    }
}

impl GlobalWorkspaceRepository for JsonWorkspaceRepository {
    fn load(&self) -> GlobalWorkspaceState {
        match std::fs::read_to_string(&self.path) {
            Ok(json_str) => match serde_json::from_str::<GlobalWorkspaceState>(&json_str) {
                Ok(state) if state.persisted.is_empty() && state.histories.is_empty() => {
                    /* WHY: Fallback to migration if the file exists but is effectively empty. */
                    self.try_migrate_v0_2_0().unwrap_or(state)
                }
                Ok(state) => state,
                Err(e) => {
                    tracing::error!(
                        "Failed to deserialize workspace state: {}. Path: {:?}",
                        e,
                        self.path
                    );
                    GlobalWorkspaceState::default()
                }
            },
            Err(e) => {
                tracing::debug!(
                    "Failed to read workspace state (might not exist yet): {}. Path: {:?}",
                    e,
                    self.path
                );

                /* WHY: Fallback migration from v0.2.0 settings.json */
                self.try_migrate_v0_2_0().unwrap_or_default()
            }
        }
    }

    fn save(&self, state: &GlobalWorkspaceState) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }

        let json_str = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.path, json_str)?;
        Ok(())
    }
}

impl JsonWorkspaceRepository {
    fn try_migrate_v0_2_0(&self) -> Option<GlobalWorkspaceState> {
        let parent = self.path.parent()?;
        let settings_path = parent.join("settings.json");

        if !settings_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&settings_path).ok()?;
        let value: serde_json::Value = serde_json::from_str(&content).ok()?;
        let workspace = value.get("workspace")?;

        let mut migrated = GlobalWorkspaceState::default();
        let mut migrated_found = false;

        if let Some(persisted) = workspace.get("persisted").and_then(|v| v.as_array()) {
            migrated.persisted = persisted
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            migrated_found = true;
        }

        if let Some(histories) = workspace.get("histories").and_then(|v| v.as_array()) {
            migrated.histories = histories
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            migrated_found = true;
        }

        if migrated_found {
            tracing::info!("Migrated workspace state from v0.2.0 settings.json");
            Some(migrated)
        } else {
            None
        }
    }
}

pub struct InMemoryWorkspaceRepository {
    state: std::sync::Mutex<GlobalWorkspaceState>,
}

impl InMemoryWorkspaceRepository {
    pub fn new(state: GlobalWorkspaceState) -> Self {
        Self {
            state: std::sync::Mutex::new(state),
        }
    }
}

impl Default for InMemoryWorkspaceRepository {
    fn default() -> Self {
        Self::new(GlobalWorkspaceState::default())
    }
}

impl GlobalWorkspaceRepository for InMemoryWorkspaceRepository {
    fn load(&self) -> GlobalWorkspaceState {
        self.state.lock().unwrap().clone()
    }

    fn save(&self, state: &GlobalWorkspaceState) -> anyhow::Result<()> {
        *self.state.lock().unwrap() = state.clone();
        Ok(())
    }

    fn is_ephemeral(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_json_repository_migration_v0_2_0() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let workspace_path = dir.path().join("workspace.json");

        /* WHY: Create a legacy settings.json matching the v0.2.0 structure */
        let legacy_json = serde_json::json!({
            "workspace": {
                "persisted": ["/path/a", "/path/b"],
                "histories": ["/path/a", "/path/c"]
            }
        });
        std::fs::write(&settings_path, legacy_json.to_string()).unwrap();

        let repo = JsonWorkspaceRepository::new(workspace_path);
        let state = repo.load();

        assert_eq!(state.persisted, vec!["/path/a", "/path/b"]);
        assert_eq!(state.histories, vec!["/path/a", "/path/c"]);
    }

    #[test]
    fn test_json_repository_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subdir/workspace.json"); /* WHY: Test parent creation */
        let repo = JsonWorkspaceRepository::new(path.clone());

        let mut state = GlobalWorkspaceState::default();
        state.persisted = vec!["/test/p1".to_string()];
        state.histories = vec!["/test/h1".to_string()];

        repo.save(&state).unwrap();
        assert!(path.exists());

        let loaded = repo.load();
        assert_eq!(loaded.persisted, state.persisted);
        assert_eq!(loaded.histories, state.histories);
    }

    #[test]
    fn test_json_repository_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.json");
        std::fs::write(&path, "not a json").unwrap();

        let repo = JsonWorkspaceRepository::new(path);
        let state = repo.load();
        /* WHY: Should return default state on deserialization error */
        assert!(state.persisted.is_empty());
        assert!(state.histories.is_empty());
    }

    #[test]
    fn test_json_repository_migration_no_data() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let workspace_path = dir.path().join("workspace.json");

        /* WHY: Create a settings.json WITHOUT workspace entry */
        let legacy_json = serde_json::json!({
            "other": "data"
        });
        std::fs::write(&settings_path, legacy_json.to_string()).unwrap();

        let repo = JsonWorkspaceRepository::new(workspace_path);
        let state = repo.load();
        assert!(state.persisted.is_empty());
    }

    #[test]
    fn test_json_repository_migration_partial_data() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let workspace_path = dir.path().join("workspace.json");

        /* WHY: Create a settings.json WITH ONLY persisted search */
        let legacy_json = serde_json::json!({
            "workspace": {
                "persisted": ["/only/one"]
            }
        });
        std::fs::write(&settings_path, legacy_json.to_string()).unwrap();

        let repo = JsonWorkspaceRepository::new(workspace_path);
        let state = repo.load();
        assert_eq!(state.persisted, vec!["/only/one"]);
        assert!(state.histories.is_empty());
    }

    #[test]
    fn test_json_repository_migration_empty_workspace_json_triggers_migration() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let workspace_path = dir.path().join("workspace.json");

        /* WHY: Create a legacy settings.json */
        let legacy_json = serde_json::json!({
            "workspace": { "persisted": ["/migrated"] }
        });
        std::fs::write(&settings_path, legacy_json.to_string()).unwrap();

        /* WHY: Create a valid but EMPTY workspace.json */
        let empty_state = GlobalWorkspaceState::default();
        std::fs::write(
            &workspace_path,
            serde_json::to_string(&empty_state).unwrap(),
        )
        .unwrap();

        let repo = JsonWorkspaceRepository::new(workspace_path);
        let state = repo.load();
        /* WHY: It should have triggered migration because the file existed but was empty */
        assert_eq!(state.persisted, vec!["/migrated"]);
    }

    #[test]
    fn test_json_repository_migration_returns_none_when_no_data_found() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let workspace_path = dir.path().join("workspace.json");

        /* WHY: Create a settings.json with workspace object but NO targeted fields */
        let legacy_json = serde_json::json!({
            "workspace": { "other": "junk" }
        });
        std::fs::write(&settings_path, legacy_json.to_string()).unwrap();

        let repo = JsonWorkspaceRepository::new(workspace_path);
        /* WHY: Direct call to check line 109 coverage branch */
        let migrated = repo.try_migrate_v0_2_0();
        assert!(migrated.is_none());
    }

    #[test]
    fn test_in_memory_repository_ephemeral() {
        let repo = InMemoryWorkspaceRepository::default();
        assert!(repo.is_ephemeral());
    }

    #[test]
    fn test_json_repository_not_ephemeral() {
        let repo = JsonWorkspaceRepository::new(PathBuf::from("dummy"));
        assert!(!repo.is_ephemeral());
    }
}
