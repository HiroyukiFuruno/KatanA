/* WHY: General-purpose caching facade for Katana.

Provides both an in-memory ephemeral cache and a persistent on-disk cache. */

mod default;
mod memory;

pub use default::DefaultCacheService;
pub use memory::InMemoryCacheService;

use serde::{Deserialize, Serialize};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

// WHY: A Facade for managing both ephemeral (in-memory) and durable (persistent) caches.
pub trait CacheFacade: Send + Sync {
    // WHY: Retrieves a value from the in-memory cache.
    fn get_memory(&self, key: &str) -> Option<String>;
    // WHY: Stores a value in the in-memory cache. Note: this does not persist across application restarts.
    fn set_memory(&self, key: &str, value: String);

    // WHY: Retrieves a value from the persistent cache.
    fn get_persistent(&self, key: &str) -> Option<String>;
    // WHY: Stores a value in the persistent cache, syncing to disk.
    #[allow(clippy::missing_errors_doc)]
    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()>;
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct PersistentData {
    pub(crate) entries: Vec<(String, String)>,
}

pub(crate) fn read_guard<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn write_guard<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(PoisonError::into_inner)
}

// WHY: Structured canonical key for persistent cache entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "namespace")]
pub enum PersistentKey {
    #[serde(rename = "workspace_tabs")]
    WorkspaceTabs { workspace_path: std::path::PathBuf },
    #[serde(rename = "diagram")]
    Diagram {
        document_path: std::path::PathBuf,
        diagram_kind: String,
        theme: String,
        source_hash: String,
    },
    #[serde(other)]
    Unknown,
}

// WHY: Envelope to hold metadata and data for per-key persistent files.
#[derive(Serialize, Deserialize)]
pub struct PersistentEntryEnvelope {
    pub storage_version: u32,
    pub key: PersistentKey,
    pub value: String,
}

impl PersistentKey {
    // WHY: Encode to a flat string for passing through CacheFacade
    pub fn to_raw_key(&self) -> Option<String> {
        match self {
            Self::WorkspaceTabs { workspace_path } => Some(format!(
                "workspace_tabs:{}",
                workspace_path.to_string_lossy()
            )),
            Self::Diagram {
                document_path,
                diagram_kind,
                theme,
                source_hash,
            } => Some(format!(
                "diagram:{}:{}:{}:{}",
                document_path.to_string_lossy(),
                diagram_kind,
                theme,
                source_hash
            )),
            Self::Unknown => None,
        }
    }

    // WHY: Decode the logical key from a raw string received by CacheFacade
    pub fn from_raw_key(raw_key: &str) -> Option<Self> {
        const MAX_TOKEN_COUNT: usize = 5;
        let parts: Vec<&str> = raw_key.splitn(MAX_TOKEN_COUNT, ':').collect();
        match parts.as_slice() {
            ["workspace_tabs", path] => Some(Self::WorkspaceTabs {
                workspace_path: std::path::PathBuf::from(path),
            }),
            ["diagram", doc_path, kind, theme, hash] => Some(Self::Diagram {
                document_path: std::path::PathBuf::from(doc_path),
                diagram_kind: kind.to_string(),
                theme: theme.to_string(),
                source_hash: hash.to_string(),
            }),
            _ => None,
        }
    }

    // WHY: Derive a deterministic, safe filename for the entry
    pub fn target_filename(&self) -> Option<String> {
        match self {
            Self::WorkspaceTabs { .. } => {
                // For workspace_tabs, hash the raw key for safety against special path chars
                let raw = self.to_raw_key()?;
                Some(format!(
                    "workspace_tabs_{:x}.json",
                    deterministic_hash(&raw)
                ))
            }
            Self::Diagram { .. } => {
                let raw = self.to_raw_key()?;
                Some(format!("diagram_{:x}.json", deterministic_hash(&raw)))
            }
            Self::Unknown => None,
        }
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

// WHY: Ensures file names are stable across runs and Rust toolchains (unlike DefaultHasher)
fn deterministic_hash(data: &str) -> u64 {
    // We can use fnv or simple dbj2 if no external crate is guaranteed,
    // but a basic dbj2 is enough for filename uniqueness here, or we can use crc32 if available.
    // Let's implement a quick FNV-1a 64-bit for zero dependency deterministic hashing.
    let mut hash: u64 = FNV_OFFSET_BASIS;
    for b in data.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_uncovered_lines_test() {
        let key = PersistentKey::Diagram {
            document_path: std::path::PathBuf::from("/a/b/c.md"),
            diagram_kind: "mermaid".to_string(),
            theme: "dark".to_string(),
            source_hash: "123".to_string(),
        };
        let raw = key.to_raw_key().unwrap();
        assert_eq!(raw, "diagram:/a/b/c.md:mermaid:dark:123");

        let decoded = PersistentKey::from_raw_key(&raw).unwrap();
        match decoded {
            PersistentKey::Diagram {
                document_path,
                diagram_kind,
                theme,
                source_hash,
            } => {
                assert_eq!(document_path.to_str().unwrap(), "/a/b/c.md");
                assert_eq!(diagram_kind, "mermaid");
                assert_eq!(theme, "dark");
                assert_eq!(source_hash, "123");
            }
            _ => panic!("Wrong type"),
        }

        let fname = key.target_filename().unwrap();
        assert!(fname.starts_with("diagram_"));
        assert!(fname.ends_with(".json"));

        assert_eq!(PersistentKey::Unknown.to_raw_key(), None);
        assert_eq!(PersistentKey::Unknown.target_filename(), None);
        assert!(matches!(
            PersistentKey::from_raw_key("invalid:format:string"),
            None
        ));
        assert!(matches!(PersistentKey::from_raw_key("invalid"), None));
    }
}
