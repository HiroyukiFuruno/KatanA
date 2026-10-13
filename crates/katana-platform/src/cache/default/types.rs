use parking_lot::RwLock;
use std::path::PathBuf;

/* WHY: The default implementation of the CacheFacade using a per-key file store for persistence. */
pub struct DefaultCacheService {
    pub(super) memory: RwLock<Vec<(String, String)>>,
    pub(super) persistent_base_path: PathBuf,
    pub(super) persistent: RwLock<Vec<(String, String)>>,
}

pub struct PlatformCachePathResolver;

impl PlatformCachePathResolver {
    pub fn cache_root() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or(PathBuf::from("."))
            .join("KatanA")
    }

    pub fn cache_json_path() -> PathBuf {
        Self::cache_root().join("cache.json")
    }
}
