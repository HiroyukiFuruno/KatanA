use katana_core::workspace::Workspace;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct WorkspaceState {
    pub data: Option<Workspace>,
    pub(crate) workspace_revision: u64,
    pub cancel_token: Option<Arc<AtomicBool>>,
    pub is_loading: bool,
    pub expanded_directories: HashSet<PathBuf>,
    pub in_memory_dirs: HashSet<PathBuf>,
    pub temporary_roots: HashSet<PathBuf>,
    pub force_tree_open: Option<bool>,
    pub flat_views: Vec<(PathBuf, bool)>,
    pub scroll_to_workspace_tab: Option<PathBuf>,
    pub(crate) explorer_projection: super::explorer_projection::ExplorerProjectionCache,
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
            workspace_revision: 0,
            cancel_token: None,
            is_loading: false,
            expanded_directories: HashSet::new(),
            in_memory_dirs: HashSet::new(),
            temporary_roots: HashSet::new(),
            force_tree_open: None,
            flat_views: Vec::new(),
            scroll_to_workspace_tab: None,
            explorer_projection: super::explorer_projection::ExplorerProjectionCache::default(),
        }
    }

    pub(crate) fn set_data(&mut self, data: Option<Workspace>) {
        self.data = data;
        self.workspace_revision = self.workspace_revision.wrapping_add(1);
    }

    pub(crate) fn revision(&self) -> u64 {
        self.workspace_revision
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

    pub fn is_temporary_root(&self, workspace_root: &Path) -> bool {
        self.temporary_roots.contains(workspace_root)
    }
}
