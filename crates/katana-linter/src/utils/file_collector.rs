use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct LinterFileOps;

impl LinterFileOps {
    pub fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(root)
            .standard_filters(true)
            .require_git(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "rs")
                && !path.components().any(|c| c.as_os_str() == "tests")
            {
                files.push(path.to_path_buf());
            }
        }

        files.sort();
        files
    }

    /* WHY: Cargo build scripts at `crates/<name>/build.rs` sit outside `src/` and are skipped
     * by `collect_rs_files`. They still spawn external processes (rustc, git) at build time,
     * so the `no-direct-process-command` lint must scan them. We walk the workspace and
     * collect every `build.rs` directly under a workspace member directory. */
    pub fn collect_build_scripts(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let crates_dir = root.join("crates");
        if !crates_dir.exists() {
            return files;
        }
        let walker = WalkBuilder::new(&crates_dir)
            .standard_filters(true)
            .require_git(false)
            .max_depth(Some(2))
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() && path.file_name().is_some_and(|n| n == "build.rs") {
                files.push(path.to_path_buf());
            }
        }

        files.sort();
        files
    }

    pub fn collect_files_by_extension(root: &Path, extension: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(root)
            .standard_filters(true)
            .require_git(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == extension) {
                files.push(path.to_path_buf());
            }
        }
        files.sort();
        files
    }

    pub fn workspace_root() -> Result<&'static Path, String> {
        use std::sync::OnceLock;
        static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
        let root = ROOT.get_or_init(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|it| it.parent())
                .map(|it| it.to_path_buf())
        });

        match root.as_ref() {
            Some(path) => Ok(path.as_path()),
            None => Err("Workspace root not found".to_string()),
        }
    }
}
