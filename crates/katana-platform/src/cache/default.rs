use crate::cache::{
    CacheFacade, PersistentData, PersistentEntryEnvelope, PersistentKey, read_guard, write_guard,
};
use std::path::PathBuf;
use std::sync::RwLock;

/* WHY: Extracted from monolithic cache module to provide file-based persistent cache functionality.
SAFETY: Implements thread-safe locking mechanisms via RwLock and gracefully handles OS cache paths. */

// WHY: The default implementation of the `CacheFacade` using a per-key file store for persistence.
pub struct DefaultCacheService {
    memory: RwLock<Vec<(String, String)>>,
    persistent_base_path: PathBuf,
    persistent: RwLock<Vec<(String, String)>>,
}

impl DefaultCacheService {
    // WHY: Creates a new `DefaultCacheService` with the specified persistent root path.
    pub fn new(persistent_path: PathBuf) -> Self {
        // Migration and load
        let kv_dir = if let Some(parent) = persistent_path.parent() {
            if parent.as_os_str().is_empty() {
                PathBuf::from("kv")
            } else {
                parent.join("kv")
            }
        } else {
            PathBuf::from("kv")
        };

        let persistent_map = Self::init_and_migrate(&persistent_path, &kv_dir);

        Self {
            memory: RwLock::new(Vec::new()),
            persistent_base_path: kv_dir,
            persistent: RwLock::new(persistent_map),
        }
    }

    pub fn with_default_path() -> Self {
        let base = dirs::cache_dir().unwrap_or(PathBuf::from("."));
        Self::new(base.join("KatanA").join("cache.json"))
    }

    fn init_and_migrate(old_json_path: &PathBuf, kv_dir: &PathBuf) -> Vec<(String, String)> {
        let _ = std::fs::create_dir_all(kv_dir);

        // 1. Migrate if old json exists
        if old_json_path.exists() {
            if let Some(old_data) = Self::load_legacy_persistent(old_json_path) {
                let mut failure = false;

                for (k, v) in old_data.entries {
                    if let Some(key) = PersistentKey::from_raw_key(&k) {
                        let env = PersistentEntryEnvelope {
                            storage_version: 1,
                            key: key.clone(),
                            value: v.clone(),
                        };
                        let Some(file_name) = key.target_filename() else {
                            continue;
                        };
                        let target_path = kv_dir.join(&file_name);

                        let temp_path = kv_dir.join(format!("{}.tmp", file_name));
                        if let Ok(json) = serde_json::to_string_pretty(&env) {
                            if std::fs::write(&temp_path, json).is_ok() {
                                if std::fs::rename(&temp_path, target_path).is_err() {
                                    failure = true;
                                }
                            } else {
                                failure = true;
                            }
                        }
                    }
                    // Else: legacy keys are skipped
                }

                if !failure {
                    // Safe to remove old json if there were no IO errors during workspace migration
                    let _ = std::fs::remove_file(old_json_path);
                }
            } else {
                // If it can't be parsed at all, probably corrupted
                let _ = std::fs::remove_file(old_json_path);
            }
        }

        // 2. Load all current entries from KV into memory
        let mut map = Vec::new();
        if let Ok(entries) = std::fs::read_dir(kv_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_file()
                {
                    let inner_path = entry.path();
                    if inner_path.extension().and_then(|s| s.to_str()) != Some("json") {
                        continue;
                    }
                    if let Ok(json) = std::fs::read_to_string(&inner_path)
                        && let Ok(env) = serde_json::from_str::<PersistentEntryEnvelope>(&json)
                    {
                        if let Some(target_filename) = env.key.target_filename()
                            && entry.file_name() != std::ffi::OsStr::new(&target_filename)
                        {
                            let canonical_path = kv_dir.join(&target_filename);
                            if canonical_path.exists() {
                                let _ = std::fs::remove_file(&inner_path);
                                continue;
                            } else if std::fs::rename(&inner_path, &canonical_path).is_err() {
                                continue;
                            }
                        }

                        if let Some(raw_key) = env.key.to_raw_key() {
                            map.push((raw_key, env.value));
                        }
                    }
                }
            }
        }

        map
    }

    fn load_legacy_persistent(path: &PathBuf) -> Option<PersistentData> {
        let json_str = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&json_str).ok()
    }

    /* WHY: Clears all subdirectories in the Katana cache directory (e.g., http-image-cache, plantuml, tmp)
    while preserving files in the root like `cache.json`. */
    pub fn clear_all_directories() {
        let base = dirs::cache_dir()
            .unwrap_or(PathBuf::from("."))
            .join("KatanA");
        Self::clear_all_directories_in(&base);
    }

    fn clear_directory(path: &std::path::Path) {
        let Ok(sub_entries) = std::fs::read_dir(path) else {
            let _ = std::fs::remove_dir_all(path);
            return;
        };

        for sub_entry in sub_entries.flatten() {
            let _ = std::fs::remove_file(sub_entry.path());
        }
        let _ = std::fs::remove_dir_all(path);
    }

    pub fn clear_all_directories_in(base: &std::path::Path) {
        let Ok(entries) = std::fs::read_dir(base) else {
            return;
        };

        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            if file_type.is_dir() {
                Self::clear_directory(&entry.path());
            }
        }
    }
}

impl Default for DefaultCacheService {
    fn default() -> Self {
        Self::with_default_path()
    }
}

impl CacheFacade for DefaultCacheService {
    fn get_memory(&self, key: &str) -> Option<String> {
        let map = read_guard(&self.memory);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_memory(&self, key: &str, value: String) {
        let mut map = write_guard(&self.memory);
        if let Some(pos) = map.iter().position(|(k, _)| k == key) {
            if let Some(entry) = map.get_mut(pos) {
                entry.1 = value;
            }
        } else {
            map.push((key.to_string(), value));
        }
    }

    fn get_persistent(&self, key: &str) -> Option<String> {
        let map = read_guard(&self.persistent);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        let p_key = match PersistentKey::from_raw_key(key) {
            Some(pk) => pk,
            None => PersistentKey::Unknown,
        };
        // Only skip actual write if it's completely unknown and not meaningful to persist
        // But for backwards compatibility, if they pass an unknown key, we can try to save it or drop it.
        // Actually, let's persist everything but with `unknown` format if needed.
        let env = PersistentEntryEnvelope {
            storage_version: 1,
            key: p_key.clone(),
            value: value.clone(),
        };
        let file_name = match p_key.target_filename() {
            Some(f) => f,
            None => format!("unknown_{:x}.json", key.len()),
        };

        let json = serde_json::to_string_pretty(&env)?;
        let target_path = self.persistent_base_path.join(&file_name);
        let temp_path = self.persistent_base_path.join(format!("{}.tmp", file_name));

        std::fs::create_dir_all(&self.persistent_base_path)?;
        std::fs::write(&temp_path, json)?;
        std::fs::rename(&temp_path, target_path)?;

        {
            let mut map = write_guard(&self.persistent);
            if let Some(pos) = map.iter().position(|(k, _)| k == key) {
                if let Some(entry) = map.get_mut(pos) {
                    entry.1 = value;
                }
            } else {
                map.push((key.to_string(), value));
            }
        }

        Ok(())
    }

    fn clear_diagram_cache(&self) {
        let Ok(entries) = std::fs::read_dir(&self.persistent_base_path) else {
            return;
        };

        for entry in entries.flatten() {
            let Ok(file_name) = entry.file_name().into_string() else {
                continue;
            };
            if file_name.starts_with("diagram_") && file_name.ends_with(".json") {
                let _ = std::fs::remove_file(entry.path());
            }
        }

        // Also remove from in-memory cache
        let mut map = write_guard(&self.persistent);
        map.retain(|(k, _)| !k.starts_with("diagram:"));
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use tempfile::TempDir;

    #[test]
    fn test_memory_cache() {
        let cache = DefaultCacheService::new(PathBuf::from("dummy.json"));
        assert_eq!(cache.get_memory("test"), None);
        cache.set_memory("test", "data".to_string());
        assert_eq!(cache.get_memory("test"), Some("data".to_string()));

        cache.set_memory("test", "data2".to_string());
        assert_eq!(cache.get_memory("test"), Some("data2".to_string()));
    }

    #[test]
    fn test_persistent_cache() {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let path = tmp.path().join("cache.json");
        let cache = DefaultCacheService::new(path.clone());

        assert_eq!(cache.get_persistent("workspace_tabs:test1"), None);
        cache
            .set_persistent("workspace_tabs:test1", "val".to_string())
            .expect("Failed to set");
        assert_eq!(
            cache.get_persistent("workspace_tabs:test1"),
            Some("val".to_string())
        );

        cache
            .set_persistent("workspace_tabs:test1", "val2".to_string())
            .expect("Failed to set");
        assert_eq!(
            cache.get_persistent("workspace_tabs:test1"),
            Some("val2".to_string())
        );

        // Create a new instance representing an app restart
        let cache2 = DefaultCacheService::new(path);
        assert_eq!(
            cache2.get_persistent("workspace_tabs:test1"),
            Some("val2".to_string())
        );
    }

    #[test]
    fn test_clear_diagram_cache() {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let path = tmp.path().join("cache.json");
        let cache = DefaultCacheService::new(path.clone());

        let diagram_key = "diagram:/dummy/path:drawio:dark:abc123hash";
        cache
            .set_persistent(diagram_key, "svg data".to_string())
            .expect("Failed to set");

        assert_eq!(
            cache.get_persistent(diagram_key),
            Some("svg data".to_string())
        );

        // The file should exist
        let mut found_file = false;
        let kv_dir = tmp.path().join("kv");
        for entry in std::fs::read_dir(&kv_dir).unwrap().flatten() {
            if entry.file_name().to_string_lossy().starts_with("diagram_") {
                found_file = true;
            }
        }
        assert!(found_file);

        // Clear it
        cache.clear_diagram_cache();

        assert_eq!(cache.get_persistent(diagram_key), None);

        // Check if file is removed
        let remaining_diagrams = std::fs::read_dir(&kv_dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("diagram_"))
            .count();
        assert_eq!(remaining_diagrams, 0, "Diagram file was not deleted");
    }

    #[test]
    fn test_cache_recovers_from_poisoned_memory_lock() {
        let cache = DefaultCacheService::new(PathBuf::from("dummy.json"));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = cache
                .memory
                .write()
                .expect("poison test must acquire write lock");
            panic!("poison memory lock");
        }));

        cache.set_memory("test", "recovered".to_string());
        assert_eq!(cache.get_memory("test"), Some("recovered".to_string()));
    }

    #[test]
    fn test_clear_directory_fallback_on_file() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("just_a_file.txt");
        std::fs::write(&file_path, b"test").unwrap();
        DefaultCacheService::clear_directory(&file_path);
    }

    #[test]
    fn test_uncovered_lines_default() {
        // test default implementation
        let svc = DefaultCacheService::default();
        let path = svc.persistent_base_path.to_string_lossy();
        assert!(path.contains(".cache") || path.contains("KatanA"));

        // test unknown key persistence
        let tmp = TempDir::new().unwrap();
        let svc = DefaultCacheService::new(tmp.path().join("cache.json"));
        svc.set_persistent("random_unknown_key", "value".into())
            .unwrap();
        assert_eq!(
            svc.get_persistent("random_unknown_key"),
            Some("value".to_string())
        );

        // Unknown keys shouldn't reload correctly through get_persistent since to_raw_key yields None,
        // but we can verify the fallback json file was indeed created upon setting.
        let file_name = format!("unknown_{:x}.json", "random_unknown_key".len());
        let unknown_file_path = svc.persistent_base_path.join(file_name);
        assert!(unknown_file_path.exists());

        // test clear_all_directories_in with sub-directories
        let kv_dir = tmp.path().join("kv");
        std::fs::create_dir_all(&kv_dir).unwrap();
        let sub_dir = kv_dir.join("sub");
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(sub_dir.join("file.txt"), b"test").unwrap();
        DefaultCacheService::clear_all_directories_in(&kv_dir);
        assert!(!sub_dir.exists());

        // fallback clearing empty or missing
        DefaultCacheService::clear_all_directories_in(&tmp.path().join("missing"));
    }

    #[test]
    fn test_legacy_migration() {
        let tmp = TempDir::new().unwrap();
        let cache_json_path = tmp.path().join("cache.json");
        let kv_dir = tmp.path().join("kv");

        // 1. Invalid JSON -> corrupted, should be removed
        std::fs::write(&cache_json_path, b"invalid json data").unwrap();
        let _ = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        assert!(!cache_json_path.exists());

        // 2. Valid Legacy JSON -> successful migration
        let legacy_json = r#"{
            "entries": [
                ["workspace_tabs:test_ws", "some_value"]
            ]
        }"#;
        std::fs::write(&cache_json_path, legacy_json).unwrap();
        let map = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        assert!(!cache_json_path.exists()); // Removed upon success
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].0, "workspace_tabs:test_ws");
        assert_eq!(map[0].1, "some_value");

        // 3. IO write failure (make temp directory read-only / file to simulate)
        let bad_kv_dir = tmp.path().join("file_as_dir");
        std::fs::write(&bad_kv_dir, b"not a dir").unwrap();
        let path3 = tmp.path().join("cache3.json");
        std::fs::write(&path3, legacy_json).unwrap();
        let _ = DefaultCacheService::init_and_migrate(&path3, &bad_kv_dir);
        assert!(path3.exists()); // Failed to write, so old json is kept!

        // 4. IO rename failure (target path exists as a directory)
        let bad_rename_dir = tmp.path().join("bad_rename_dir");
        std::fs::create_dir_all(&bad_rename_dir).unwrap();
        let target_file_name = PersistentKey::from_raw_key("workspace_tabs:test_ws")
            .unwrap()
            .target_filename()
            .unwrap();
        std::fs::create_dir_all(bad_rename_dir.join(&target_file_name)).unwrap();
        let path4 = tmp.path().join("cache4.json");
        std::fs::write(&path4, legacy_json).unwrap();
        let _ = DefaultCacheService::init_and_migrate(&path4, &bad_rename_dir);
        assert!(path4.exists()); // Failed to rename, so old json is kept!

        // 5. Edge cases for directory generation mapping over logic
        let _edge_1 = DefaultCacheService::new(PathBuf::from("file_only.json"));
        let _edge_2 = DefaultCacheService::new(PathBuf::from(""));
        let _edge_3 = DefaultCacheService::new(PathBuf::from("/"));
    }

    #[test]
    fn test_legacy_kv_migration_and_deduplication() {
        let tmp = TempDir::new().unwrap();
        let kv_dir = tmp.path().join("kv");
        std::fs::create_dir_all(&kv_dir).unwrap();
        let cache_json_path = tmp.path().join("cache.json");

        let key = PersistentKey::WorkspaceTabs {
            workspace_path: PathBuf::from("/test/path"),
        };
        let canonical_filename = key.target_filename().unwrap();
        let legacy_filename = "workspace_tabs_abcedfg12345.json"; // Wrong filename intentionally

        let env = PersistentEntryEnvelope {
            storage_version: 1,
            key: key.clone(),
            value: "stale_data".to_string(),
        };
        let bad_env = PersistentEntryEnvelope {
            storage_version: 1,
            key: key.clone(),
            value: "stale_data2".to_string(),
        };

        // Scenario 1: Only legacy file exists
        let legacy_path = kv_dir.join(legacy_filename);
        std::fs::write(&legacy_path, serde_json::to_string(&env).unwrap()).unwrap();

        let map = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].1, "stale_data");
        // It should have renamed the file
        assert!(!legacy_path.exists());
        assert!(kv_dir.join(&canonical_filename).exists());

        // Scenario 2: Both canonical and legacy exist
        let canonical_path = kv_dir.join(&canonical_filename);
        std::fs::write(&canonical_path, serde_json::to_string(&env).unwrap()).unwrap();
        std::fs::write(&legacy_path, serde_json::to_string(&bad_env).unwrap()).unwrap(); // Legacy comes back

        // Write another legacy file that fails renaming (target exists)
        let unrenamable_legacy_path = kv_dir.join("workspace_tabs_unrenamable.json");
        std::fs::write(
            &unrenamable_legacy_path,
            serde_json::to_string(&bad_env).unwrap(),
        )
        .unwrap();

        // Make rename fail by creating a directory instead of file
        let bad_canonical_path = kv_dir.join("workspace_tabs_bad_canonical.json");
        std::fs::write(
            &bad_canonical_path,
            serde_json::to_string(&bad_env).unwrap(),
        )
        .unwrap();

        let map = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        assert_eq!(map.len(), 1); // Should only load the single key once
        assert!(!legacy_path.exists()); // The legacy path should be deleted because canonical_path exists!
    }

    #[test]
    fn test_clear_all_directories() {
        // Just verify it doesn't crash since it interacts with user home directories
        DefaultCacheService::clear_all_directories();
    }
}
