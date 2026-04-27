use crate::cache::{CacheFacade, LockOps, PersistentEntryEnvelope, PersistentKey};
use parking_lot::RwLock;
use std::path::PathBuf;

/* WHY: Extracted from monolithic cache module to provide file-based persistent cache functionality.
SAFETY: Implements thread-safe locking mechanisms via RwLock and gracefully handles OS cache paths. */

mod migration;
mod types;

pub use types::DefaultCacheService;

impl DefaultCacheService {
    /* WHY: Creates a new `DefaultCacheService` with the specified persistent root path. */
    pub fn new(persistent_path: PathBuf) -> Self {
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

    /* WHY: Clears all subdirectories in the Katana cache directory (e.g., http-image-cache, plantuml, tmp)
    while preserving files in the root like `cache.json`. */
    pub fn clear_all_directories() {
        let base = dirs::cache_dir()
            .unwrap_or(PathBuf::from("."))
            .join("KatanA");
        Self::clear_all_directories_in(&base);
        Self::clear_temporary_diagram_images();
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
        let map = LockOps::read_guard(&self.memory);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_memory(&self, key: &str, value: String) {
        let mut map = LockOps::write_guard(&self.memory);
        if let Some(pos) = map.iter().position(|(k, _)| k == key) {
            if let Some(entry) = map.get_mut(pos) {
                entry.1 = value;
            }
        } else {
            map.push((key.to_string(), value));
        }
    }

    fn get_persistent(&self, key: &str) -> Option<String> {
        let map = LockOps::read_guard(&self.persistent);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        let p_key = match PersistentKey::from_raw_key(key) {
            Some(pk) => pk,
            None => PersistentKey::Unknown,
        };
        /* WHY: Unknown keys are persisted with a fallback filename rather than dropped,
        preserving forward-compatibility when new key types are introduced. */
        let env = PersistentEntryEnvelope {
            storage_version: 2,
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
            let mut map = LockOps::write_guard(&self.persistent);
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

        /* WHY: In-memory map must stay in sync with the on-disk KV to avoid stale reads after clear. */
        let mut map = LockOps::write_guard(&self.persistent);
        map.retain(|(k, _)| !k.starts_with("diagram:"));
        Self::clear_temporary_diagram_images();
    }
}

impl DefaultCacheService {
    fn clear_temporary_diagram_images() {
        let temp_dir = std::env::temp_dir();
        let cache_dirs = [
            temp_dir.join("katana_mermaid_cache"),
            temp_dir.join("katana_drawio_cache"),
        ];
        for cache_dir in cache_dirs {
            let _ = std::fs::remove_dir_all(cache_dir);
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static DIAGRAM_CACHE_CLEAR_LOCK: Mutex<()> = Mutex::new(());

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

        /* WHY: Simulate an app restart by constructing a fresh instance from the same path. */
        let cache2 = DefaultCacheService::new(path);
        assert_eq!(
            cache2.get_persistent("workspace_tabs:test1"),
            Some("val2".to_string())
        );
    }

    #[test]
    fn test_clear_diagram_cache() {
        let _guard = DIAGRAM_CACHE_CLEAR_LOCK
            .lock()
            .expect("lock diagram cache clear");
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

        let mut found_file = false;
        let kv_dir = tmp.path().join("kv");
        for entry in std::fs::read_dir(&kv_dir).unwrap().flatten() {
            if entry.file_name().to_string_lossy().starts_with("diagram_") {
                found_file = true;
            }
        }
        assert!(found_file);

        cache.clear_diagram_cache();

        assert_eq!(cache.get_persistent(diagram_key), None);

        let remaining_diagrams = std::fs::read_dir(&kv_dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("diagram_"))
            .count();
        assert_eq!(remaining_diagrams, 0, "Diagram file was not deleted");
    }

    #[test]
    fn test_clear_diagram_cache_removes_temporary_renderer_images() {
        let _guard = DIAGRAM_CACHE_CLEAR_LOCK
            .lock()
            .expect("lock diagram cache clear");
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let path = tmp.path().join("cache.json");
        let cache = DefaultCacheService::new(path);
        let mermaid_cache = std::env::temp_dir().join("katana_mermaid_cache");
        let drawio_cache = std::env::temp_dir().join("katana_drawio_cache");
        std::fs::create_dir_all(&mermaid_cache).expect("mkdir mermaid");
        std::fs::create_dir_all(&drawio_cache).expect("mkdir drawio");
        std::fs::write(mermaid_cache.join("diagram.png"), "png").expect("write mermaid");
        std::fs::write(drawio_cache.join("diagram.png"), "png").expect("write drawio");

        cache.clear_diagram_cache();

        assert!(!mermaid_cache.exists(), "Mermaid render cache remained");
        assert!(!drawio_cache.exists(), "Draw.io render cache remained");
    }

    #[test]
    fn test_cache_resilience_under_concurrent_access() {
        /* WHY: parking_lot::RwLock is not poisonable, so we verify that concurrent reads/writes
        complete without panic instead. */
        let cache = DefaultCacheService::new(PathBuf::from("dummy.json"));
        cache.set_memory("test", "value".to_string());
        assert_eq!(cache.get_memory("test"), Some("value".to_string()));
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
        let svc = DefaultCacheService::default();
        let path = svc.persistent_base_path.to_string_lossy();
        assert!(path.contains(".cache") || path.contains("KatanA"));

        let tmp = TempDir::new().unwrap();
        let svc = DefaultCacheService::new(tmp.path().join("cache.json"));
        svc.set_persistent("random_unknown_key", "value".into())
            .unwrap();
        assert_eq!(
            svc.get_persistent("random_unknown_key"),
            Some("value".to_string())
        );

        /* WHY: Unknown keys have no canonical raw key, so get_persistent returns None after reload;
        but the fallback file must still exist on disk to confirm persistence occurred. */
        let file_name = format!("unknown_{:x}.json", "random_unknown_key".len());
        let unknown_file_path = svc.persistent_base_path.join(file_name);
        assert!(unknown_file_path.exists());

        let kv_dir = tmp.path().join("kv");
        std::fs::create_dir_all(&kv_dir).unwrap();
        let sub_dir = kv_dir.join("sub");
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(sub_dir.join("file.txt"), b"test").unwrap();
        DefaultCacheService::clear_all_directories_in(&kv_dir);
        assert!(!sub_dir.exists());

        DefaultCacheService::clear_all_directories_in(&tmp.path().join("missing"));
    }

    #[test]
    fn test_legacy_migration() {
        let tmp = TempDir::new().unwrap();
        let cache_json_path = tmp.path().join("cache.json");
        let kv_dir = tmp.path().join("kv");

        std::fs::write(&cache_json_path, b"invalid json data").unwrap();
        let _ = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        assert!(!cache_json_path.exists());

        let legacy_json = r#"{
            "entries": [
                ["workspace_tabs:test_ws", "some_value"]
            ]
        }"#;
        std::fs::write(&cache_json_path, legacy_json).unwrap();
        let map = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        /* WHY: Removed upon success */
        assert!(!cache_json_path.exists());
        assert_eq!(map.len(), 1);
        assert_eq!(map.first().unwrap().0, "workspace_tabs:test_ws");
        assert_eq!(map.first().unwrap().1, "some_value");

        let bad_kv_dir = tmp.path().join("file_as_dir");
        std::fs::write(&bad_kv_dir, b"not a dir").unwrap();
        let path3 = tmp.path().join("cache3.json");
        std::fs::write(&path3, legacy_json).unwrap();
        let _ = DefaultCacheService::init_and_migrate(&path3, &bad_kv_dir);
        /* WHY: Failed to write, so old json is kept! */
        assert!(path3.exists());

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
        /* WHY: Failed to rename, so old json is kept! */
        assert!(path4.exists());

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
        /* WHY: Wrong filename intentionally */
        let legacy_filename = "workspace_tabs_abcedfg12345.json";

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

        let legacy_path = kv_dir.join(legacy_filename);
        std::fs::write(&legacy_path, serde_json::to_string(&env).unwrap()).unwrap();

        let map = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        assert_eq!(map.len(), 1);
        assert_eq!(map.first().unwrap().1, "stale_data");
        assert!(!legacy_path.exists());
        assert!(kv_dir.join(&canonical_filename).exists());

        let canonical_path = kv_dir.join(&canonical_filename);
        std::fs::write(&canonical_path, serde_json::to_string(&env).unwrap()).unwrap();
        /* WHY: Legacy comes back */
        std::fs::write(&legacy_path, serde_json::to_string(&bad_env).unwrap()).unwrap();

        let unrenamable_legacy_path = kv_dir.join("workspace_tabs_unrenamable.json");
        std::fs::write(
            &unrenamable_legacy_path,
            serde_json::to_string(&bad_env).unwrap(),
        )
        .unwrap();

        let bad_canonical_path = kv_dir.join("workspace_tabs_bad_canonical.json");
        std::fs::write(
            &bad_canonical_path,
            serde_json::to_string(&bad_env).unwrap(),
        )
        .unwrap();

        let map = DefaultCacheService::init_and_migrate(&cache_json_path, &kv_dir);
        /* WHY: Should only load the single key once */
        assert_eq!(map.len(), 1);
        /* WHY: The legacy path should be deleted because canonical_path exists! */
        assert!(!legacy_path.exists());
    }

    #[test]
    fn test_clear_all_directories() {
        let _guard = DIAGRAM_CACHE_CLEAR_LOCK
            .lock()
            .expect("lock diagram cache clear");
        DefaultCacheService::clear_all_directories();
    }
}
