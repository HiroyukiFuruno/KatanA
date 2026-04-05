use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct PersistentData {
    pub(crate) entries: Vec<(String, String)>,
}

pub(crate) struct LockOps;

impl LockOps {
    pub(crate) fn read_guard<T>(lock: &parking_lot::RwLock<T>) -> parking_lot::RwLockReadGuard<'_, T> {
        lock.read()
    }

    pub(crate) fn write_guard<T>(lock: &parking_lot::RwLock<T>) -> parking_lot::RwLockWriteGuard<'_, T> {
        lock.write()
    }
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
            Self::WorkspaceTabs { workspace_path } => {
                let mut path_str = workspace_path.to_string_lossy().to_string();
                if path_str.ends_with('/') || path_str.ends_with('\\') {
                    path_str.pop();
                }
                Some(format!("workspace_tabs:{}", path_str))
            }
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
            ["workspace_tabs", path] => {
                let mut p = path.to_string();
                if p.ends_with('/') || p.ends_with('\\') {
                    p.pop();
                }
                Some(Self::WorkspaceTabs {
                    workspace_path: std::path::PathBuf::from(p),
                })
            }
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
                let raw = self.to_raw_key()?;
                Some(format!(
                    "workspace_tabs_{:x}.json",
                    CacheUtils::deterministic_hash(&raw)
                ))
            }
            Self::Diagram { .. } => {
                let raw = self.to_raw_key()?;
                Some(format!(
                    "diagram_{:x}.json",
                    CacheUtils::deterministic_hash(&raw)
                ))
            }
            Self::Unknown => None,
        }
    }
}

pub struct CacheUtils;

impl CacheUtils {
    // WHY: Ensures file names are stable across runs and Rust toolchains (unlike DefaultHasher)
    pub fn deterministic_hash(data: &str) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;
        let mut hash: u64 = FNV_OFFSET_BASIS;
        for b in data.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}
