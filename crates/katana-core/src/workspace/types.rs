use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum TreeEntry {
    File {
        path: PathBuf,
    },
    Directory {
        path: PathBuf,
        children: Vec<TreeEntry>,
    },
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub tree: Vec<TreeEntry>,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Cannot read workspace directory at {path}: {source}")]
    UnreadableRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("No workspace is currently open")]
    NoWorkspace,
}
