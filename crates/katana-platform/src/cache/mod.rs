use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;

pub trait CacheFacade: Send + Sync {
    fn get_memory(&self, key: &str) -> Option<String>;
    fn get_persistent(&self, key: &str) -> Option<String>;
    fn set_memory(&self, key: &str, value: String);
    /// # Errors
    /// Returns an error if the value cannot be saved to the persistent storage.
    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()>;
    fn clear_all_directories(&self);
}

pub struct DefaultCacheService {
    memory: Mutex<HashMap<String, String>>,
}

impl DefaultCacheService {
    pub fn new() -> Self {
        Self {
            memory: Mutex::new(HashMap::new()),
        }
    }
    pub fn clear_all_directories() {
        // Static method for legacy code
    }
    pub fn clear_diagram_cache(&self) {
        // Instance method for UI code
    }
}

impl Default for DefaultCacheService {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheFacade for DefaultCacheService {
    fn get_memory(&self, key: &str) -> Option<String> {
        self.memory.lock().get(key).cloned()
    }
    fn get_persistent(&self, key: &str) -> Option<String> {
        self.memory.lock().get(key).cloned()
    }
    fn set_memory(&self, key: &str, value: String) {
        self.memory.lock().insert(key.to_string(), value);
    }
    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        self.memory.lock().insert(key.to_string(), value);
        Ok(())
    }
    fn clear_all_directories(&self) {
        self.memory.lock().clear();
    }
}

pub struct InMemoryCacheService {
    memory: Mutex<HashMap<String, String>>,
}

impl Default for InMemoryCacheService {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryCacheService {
    pub fn new() -> Self {
        Self {
            memory: Mutex::new(HashMap::new()),
        }
    }
}

impl CacheFacade for InMemoryCacheService {
    fn get_memory(&self, key: &str) -> Option<String> {
        self.memory.lock().get(key).cloned()
    }
    fn get_persistent(&self, key: &str) -> Option<String> {
        self.memory.lock().get(key).cloned()
    }
    fn set_memory(&self, key: &str, value: String) {
        self.memory.lock().insert(key.to_string(), value);
    }
    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        self.memory.lock().insert(key.to_string(), value);
        Ok(())
    }
    fn clear_all_directories(&self) {
        self.memory.lock().clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentKey {
    Diagram {
        document_path: PathBuf,
        diagram_kind: String,
        theme: String,
        source_hash: String,
    },
    WorkspaceTabs {
        workspace_path: PathBuf,
    },
}

impl PersistentKey {
    pub fn to_raw_key(&self) -> Option<String> {
        match self {
            Self::Diagram {
                document_path,
                diagram_kind,
                theme,
                source_hash,
            } => Some(format!(
                "diagram:{}:{}:{}:{}",
                document_path.display(),
                diagram_kind,
                theme,
                source_hash
            )),
            Self::WorkspaceTabs { workspace_path } => {
                Some(format!("workspace_tabs:{}", workspace_path.display()))
            }
        }
    }
}
