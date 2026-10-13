use crate::cache::{PersistentData, PersistentEntryEnvelope, PersistentKey};

use super::types::DefaultCacheService;

impl DefaultCacheService {
    pub(super) fn init_and_migrate(
        old_json_path: &std::path::Path,
        kv_dir: &std::path::Path,
    ) -> Vec<(String, String)> {
        let _ = std::fs::create_dir_all(kv_dir);
        Self::migrate_legacy_json(old_json_path, kv_dir);
        Self::load_kv_entries(kv_dir)
    }

    fn migrate_legacy_json(old_json_path: &std::path::Path, kv_dir: &std::path::Path) {
        if !old_json_path.exists() {
            return;
        }
        match Self::load_legacy_persistent(old_json_path) {
            Some(old_data) => Self::write_legacy_entries(old_json_path, kv_dir, old_data.entries),
            None => {
                /* WHY: Unparseable legacy JSON is treated as corrupted and removed to allow a clean start. */
                let _ = std::fs::remove_file(old_json_path);
            }
        }
    }

    fn write_legacy_entries(
        old_json_path: &std::path::Path,
        kv_dir: &std::path::Path,
        entries: Vec<(String, String)>,
    ) {
        let mut failure = false;
        for (k, v) in entries {
            let Some(key) = PersistentKey::from_raw_key(&k) else {
                continue;
            };
            let env = PersistentEntryEnvelope {
                storage_version: 1,
                key: key.clone(),
                value: v,
            };
            let Some(file_name) = key.target_filename() else {
                continue;
            };
            let target_path = kv_dir.join(&file_name);
            let temp_path = kv_dir.join(format!("{}.tmp", file_name));
            let Ok(json) = serde_json::to_string_pretty(&env) else {
                continue;
            };
            if std::fs::write(&temp_path, json).is_err() {
                failure = true;
                continue;
            }
            if std::fs::rename(&temp_path, target_path).is_err() {
                failure = true;
            }
        }
        if !failure {
            /* WHY: Only delete the legacy file after all IO succeeded to avoid data loss on partial migration. */
            let _ = std::fs::remove_file(old_json_path);
        }
    }

    fn load_kv_entries(kv_dir: &std::path::Path) -> Vec<(String, String)> {
        let mut map = Vec::new();
        let Ok(entries) = std::fs::read_dir(kv_dir) else {
            return map;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            Self::try_load_kv_entry(&entry, kv_dir, &mut map);
        }
        map
    }

    fn try_load_kv_entry(
        entry: &std::fs::DirEntry,
        kv_dir: &std::path::Path,
        map: &mut Vec<(String, String)>,
    ) {
        let inner_path = entry.path();
        if inner_path.extension().and_then(|s| s.to_str()) != Some("json") {
            return;
        }
        let Ok(json) = std::fs::read_to_string(&inner_path) else {
            return;
        };
        let Ok(env) = serde_json::from_str::<PersistentEntryEnvelope>(&json) else {
            return;
        };
        if Self::is_stale_mermaid(&env) {
            let _ = std::fs::remove_file(&inner_path);
            return;
        }
        if !Self::ensure_canonical_path(entry, &env, kv_dir, &inner_path) {
            return;
        }
        if let Some(raw_key) = env.key.to_raw_key() {
            map.push((raw_key, env.value));
        }
    }

    fn is_stale_mermaid(env: &PersistentEntryEnvelope) -> bool {
        if let PersistentKey::Diagram {
            ref diagram_kind, ..
        } = env.key
        {
            /* WHY: Mermaid SVG format changed in v2; old entries are stale and must be evicted. */
            return diagram_kind == "mermaid" && env.storage_version < 2;
        }
        false
    }

    fn ensure_canonical_path(
        entry: &std::fs::DirEntry,
        env: &PersistentEntryEnvelope,
        kv_dir: &std::path::Path,
        inner_path: &std::path::Path,
    ) -> bool {
        let Some(target_filename) = env.key.target_filename() else {
            return true;
        };
        if entry.file_name() == std::ffi::OsStr::new(&target_filename) {
            return true;
        }
        let canonical_path = kv_dir.join(&target_filename);
        if canonical_path.exists() {
            let _ = std::fs::remove_file(inner_path);
            return false;
        }
        if std::fs::rename(inner_path, &canonical_path).is_err() {
            return false;
        }
        true
    }

    pub(super) fn load_legacy_persistent(path: &std::path::Path) -> Option<PersistentData> {
        let json_str = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&json_str).ok()
    }
}
