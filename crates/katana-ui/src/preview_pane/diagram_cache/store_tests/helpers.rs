use katana_platform::CacheFacade;
use std::path::Path;
use std::sync::Arc;

pub(super) fn test_cache() -> (tempfile::TempDir, Arc<dyn CacheFacade>) {
    let tmp = tempfile::TempDir::new().expect("temp dir should be created");
    let cache: Arc<dyn CacheFacade> = Arc::new(katana_platform::DefaultCacheService::new(
        tmp.path().join("cache.json"),
    ));
    (tmp, cache)
}

pub(super) fn count_files_with_extension(root: &Path, extension: &str) -> usize {
    std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir() || path.extension().and_then(|it| it.to_str()) == Some(extension)
        })
        .map(|path| {
            if path.is_dir() {
                count_files_with_extension(&path, extension)
            } else {
                1
            }
        })
        .sum()
}

pub(super) fn svg_file_names(root: &Path) -> Vec<String> {
    let mut names = collect_svg_file_names(root);
    names.sort();
    names
}

fn collect_svg_file_names(root: &Path) -> Vec<String> {
    std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return collect_svg_file_names(&path);
            }
            if path.extension().and_then(|it| it.to_str()) == Some("svg") {
                let Some(file_name) = path.file_name() else {
                    return Vec::new();
                };
                return vec![file_name.to_string_lossy().to_string()];
            }
            Vec::new()
        })
        .collect()
}
