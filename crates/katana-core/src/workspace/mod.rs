use std::path::{Path, PathBuf};

mod types;
pub use types::*;

impl TreeEntry {
    pub fn path(&self) -> &Path {
        match self {
            Self::File { path } => path,
            Self::Directory { path, .. } => path,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.path().file_name()?.to_str()
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_markdown(&self) -> bool {
        match self {
            Self::File { path } => {
                let ext = path.extension();
                ext.map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn collect_all_directory_paths(&self, paths: &mut Vec<PathBuf>) {
        if let Self::Directory { path, children } = self {
            paths.push(path.clone());
            for child in children {
                child.collect_all_directory_paths(paths);
            }
        }
    }

    pub fn collect_all_markdown_file_paths(&self, paths: &mut Vec<PathBuf>) {
        match self {
            Self::File { path } => {
                if self.is_markdown() {
                    paths.push(path.clone());
                }
            }
            Self::Directory { children, .. } => {
                for child in children {
                    child.collect_all_markdown_file_paths(paths);
                }
            }
        }
    }
}

impl Workspace {
    pub fn new(root: impl Into<PathBuf>, tree: Vec<TreeEntry>) -> Self {
        Self {
            root: root.into(),
            tree,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.root.file_name()?.to_str()
    }

    pub fn collect_all_directory_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for entry in &self.tree {
            entry.collect_all_directory_paths(&mut paths);
        }
        paths
    }

    pub fn collect_all_markdown_file_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for entry in &self.tree {
            entry.collect_all_markdown_file_paths(&mut paths);
        }
        paths
    }
}

impl WorkspaceError {
    pub fn unreadable_root(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::UnreadableRoot {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests;
