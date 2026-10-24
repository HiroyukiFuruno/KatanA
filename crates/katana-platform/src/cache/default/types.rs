use parking_lot::RwLock;
use std::ffi::OsString;
use std::path::PathBuf;

const CONFIG_DIR_ENV: &str = "KATANA_CONFIG_DIR";

/* WHY: The default implementation of the CacheFacade using a per-key file store for persistence. */
pub struct DefaultCacheService {
    pub(super) memory: RwLock<Vec<(String, String)>>,
    pub(super) persistent_base_path: PathBuf,
    pub(super) persistent: RwLock<Vec<(String, String)>>,
}

pub struct PlatformCachePathResolver;

impl PlatformCachePathResolver {
    pub fn cache_root() -> PathBuf {
        resolve_cache_root(std::env::var_os(CONFIG_DIR_ENV), dirs::cache_dir())
    }

    pub fn cache_json_path() -> PathBuf {
        Self::cache_root().join("cache.json")
    }
}

fn resolve_cache_root(
    configured: Option<OsString>,
    platform_cache_dir: Option<PathBuf>,
) -> PathBuf {
    /* WHY: 隔離プロファイルの文書検査が通常利用者のキャッシュを共有しないようにする。 */
    configured
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| path.join("cache"))
        .unwrap_or_else(|| {
            platform_cache_dir
                .unwrap_or_else(|| PathBuf::from("."))
                .join("KatanA")
        })
}

#[cfg(test)]
mod tests {
    use super::resolve_cache_root;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn configured_profile_keeps_cache_inside_config_directory() {
        let root = resolve_cache_root(
            Some(OsString::from("/tmp/katana-isolated")),
            Some(PathBuf::from("/Users/test/Library/Caches")),
        );

        assert_eq!(root, PathBuf::from("/tmp/katana-isolated/cache"));
    }

    #[test]
    fn empty_config_directory_keeps_platform_cache_default() {
        let root = resolve_cache_root(
            Some(OsString::new()),
            Some(PathBuf::from("/Users/test/Library/Caches")),
        );

        assert_eq!(root, PathBuf::from("/Users/test/Library/Caches/KatanA"));
    }
}
