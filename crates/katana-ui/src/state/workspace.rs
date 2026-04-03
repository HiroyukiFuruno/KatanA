use katana_core::workspace::Workspace;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct WorkspaceState {
    pub data: Option<Workspace>,
    pub cancel_token: Option<Arc<AtomicBool>>,
    pub is_loading: bool,
    pub expanded_directories: HashSet<PathBuf>,
    pub in_memory_dirs: HashSet<PathBuf>,
    pub force_tree_open: Option<bool>,
    pub flat_views: Vec<(PathBuf, bool)>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            data: None,
            cancel_token: None,
            is_loading: false,
            expanded_directories: HashSet::new(),
            in_memory_dirs: HashSet::new(),
            force_tree_open: None,
            flat_views: Vec::new(),
        }
    }

    pub fn is_flat_view(&self, workspace_root: &Path) -> bool {
        self.flat_views
            .iter()
            .find(|(p, _)| p == workspace_root)
            .map(|(_, flat)| *flat)
            .unwrap_or(false)
    }

    pub fn set_flat_view(&mut self, workspace_root: PathBuf, flat: bool) {
        if let Some(entry) = self
            .flat_views
            .iter_mut()
            .find(|(p, _)| *p == workspace_root)
        {
            entry.1 = flat;
        } else {
            self.flat_views.push((workspace_root, flat));
        }
    }
}
