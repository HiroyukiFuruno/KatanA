use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

static NEXT_WORKSPACE_REVISION: AtomicU64 = AtomicU64::new(1);

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
    pub(crate) revision: u64,
}

impl Workspace {
    pub(crate) fn next_revision() -> u64 {
        NEXT_WORKSPACE_REVISION.fetch_add(1, Ordering::Relaxed)
    }
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
