use std::collections::HashSet;
use std::path::PathBuf;

use katana_core::workspace::{TreeEntry, Workspace};

use super::search::SearchParams;
use explorer_projection_logic::ExplorerProjectionLogicOps;

#[path = "explorer_projection_logic.rs"]
mod explorer_projection_logic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExplorerProjectionRow {
    index_path: Vec<usize>,
    pub depth: usize,
}

impl ExplorerProjectionRow {
    pub fn resolve<'a>(&self, tree: &'a [TreeEntry]) -> Option<&'a TreeEntry> {
        let (first, rest) = self.index_path.split_first()?;
        let mut entry = tree.get(*first)?;
        for index in rest {
            let TreeEntry::Directory { children, .. } = entry else {
                return None;
            };
            entry = children.get(*index)?;
        }
        Some(entry)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExplorerProjectionKey {
    workspace_revision: u64,
    expanded_directories: HashSet<PathBuf>,
    filter: Option<SearchParams>,
    is_flat_view: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ExplorerProjectionCache {
    key: Option<ExplorerProjectionKey>,
    rows: Vec<ExplorerProjectionRow>,
    rebuild_count: u64,
    estimated_heap_bytes: usize,
}

impl ExplorerProjectionCache {
    pub(crate) fn visible_row_range(
        viewport: std::ops::Range<f32>,
        row_stride: f32,
        total_rows: usize,
    ) -> std::ops::Range<usize> {
        ExplorerProjectionLogicOps::visible_row_range(viewport, row_stride, total_rows)
    }

    pub fn refresh(
        &mut self,
        workspace: &Workspace,
        expanded_directories: &HashSet<PathBuf>,
        filter: Option<(&SearchParams, &HashSet<PathBuf>)>,
        is_flat_view: bool,
    ) -> bool {
        if self.matches(workspace, expanded_directories, filter, is_flat_view) {
            return false;
        }
        let key = ExplorerProjectionKey {
            workspace_revision: workspace.revision(),
            expanded_directories: expanded_directories.clone(),
            filter: filter.map(|(params, _)| params.clone()),
            is_flat_view,
        };
        self.rows = Self::build_rows(
            &workspace.tree,
            expanded_directories,
            filter.map(|(_, paths)| paths),
            is_flat_view,
        );
        self.estimated_heap_bytes =
            Self::estimate_rows_heap_bytes(&self.rows, self.rows.capacity());
        self.key = Some(key);
        self.rebuild_count += 1;
        true
    }

    fn matches(
        &self,
        workspace: &Workspace,
        expanded_directories: &HashSet<PathBuf>,
        filter: Option<(&SearchParams, &HashSet<PathBuf>)>,
        is_flat_view: bool,
    ) -> bool {
        self.key.as_ref().is_some_and(|key| {
            key.workspace_revision == workspace.revision()
                && key.expanded_directories == *expanded_directories
                && key.filter.as_ref() == filter.map(|(params, _)| params)
                && key.is_flat_view == is_flat_view
        })
    }

    pub fn rows(&self) -> &[ExplorerProjectionRow] {
        &self.rows
    }

    pub fn estimated_heap_bytes(&self) -> usize {
        self.estimated_heap_bytes
    }

    #[cfg(test)]
    pub fn rebuild_count(&self) -> u64 {
        self.rebuild_count
    }

    fn build_rows(
        tree: &[TreeEntry],
        expanded_directories: &HashSet<PathBuf>,
        visible_paths: Option<&HashSet<PathBuf>>,
        is_flat_view: bool,
    ) -> Vec<ExplorerProjectionRow> {
        ExplorerProjectionLogicOps::build_rows(
            tree,
            expanded_directories,
            visible_paths,
            is_flat_view,
        )
    }

    fn estimate_rows_heap_bytes(rows: &[ExplorerProjectionRow], row_capacity: usize) -> usize {
        ExplorerProjectionLogicOps::estimate_rows_heap_bytes(rows, row_capacity)
    }
}

#[cfg(test)]
#[path = "explorer_projection_tests.rs"]
mod explorer_projection_tests;
