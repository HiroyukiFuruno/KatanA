use super::ExplorerProjectionRow;
use katana_core::workspace::TreeEntry;
use std::collections::HashSet;
use std::ops::Range;
use std::path::PathBuf;

pub(crate) struct ExplorerProjectionLogicOps;

impl ExplorerProjectionLogicOps {
    pub(crate) fn visible_row_range(
        viewport: Range<f32>,
        row_stride: f32,
        total_rows: usize,
    ) -> Range<usize> {
        let start = (viewport.start.max(0.0) / row_stride).floor() as usize;
        let end = (viewport.end.max(0.0) / row_stride).ceil() as usize + 1;
        start.min(total_rows)..end.min(total_rows)
    }

    pub(crate) fn build_rows(
        tree: &[TreeEntry],
        expanded_directories: &HashSet<PathBuf>,
        visible_paths: Option<&HashSet<PathBuf>>,
        is_flat_view: bool,
    ) -> Vec<ExplorerProjectionRow> {
        let mut rows = Vec::new();
        let mut index_path = Vec::new();
        if is_flat_view {
            collect_flat(tree, visible_paths, &mut index_path, &mut rows);
        } else {
            collect_tree(
                tree,
                expanded_directories,
                visible_paths,
                &mut index_path,
                0,
                &mut rows,
            );
        }
        rows
    }

    pub(crate) fn estimate_rows_heap_bytes(
        rows: &[ExplorerProjectionRow],
        row_capacity: usize,
    ) -> usize {
        row_capacity
            .saturating_mul(std::mem::size_of::<ExplorerProjectionRow>())
            .saturating_add(rows.iter().fold(0_usize, |total, row| {
                total.saturating_add(
                    row.index_path
                        .capacity()
                        .saturating_mul(std::mem::size_of::<usize>()),
                )
            }))
    }
}

fn collect_tree(
    entries: &[TreeEntry],
    expanded: &HashSet<PathBuf>,
    visible: Option<&HashSet<PathBuf>>,
    index_path: &mut Vec<usize>,
    depth: usize,
    rows: &mut Vec<ExplorerProjectionRow>,
) {
    for (index, entry) in entries.iter().enumerate() {
        if visible.is_some_and(|paths| !paths.contains(entry.path())) {
            continue;
        }
        index_path.push(index);
        rows.push(ExplorerProjectionRow {
            index_path: index_path.clone(),
            depth,
        });
        if let TreeEntry::Directory { path, children } = entry
            && expanded.contains(path)
        {
            collect_tree(children, expanded, visible, index_path, depth + 1, rows);
        }
        index_path.pop();
    }
}

fn collect_flat(
    entries: &[TreeEntry],
    visible: Option<&HashSet<PathBuf>>,
    index_path: &mut Vec<usize>,
    rows: &mut Vec<ExplorerProjectionRow>,
) {
    for (index, entry) in entries.iter().enumerate() {
        index_path.push(index);
        match entry {
            TreeEntry::File { path } if visible.is_none_or(|paths| paths.contains(path)) => {
                rows.push(ExplorerProjectionRow {
                    index_path: index_path.clone(),
                    depth: 0,
                });
            }
            TreeEntry::Directory { children, .. } => {
                collect_flat(children, visible, index_path, rows);
            }
            TreeEntry::File { .. } => {}
        }
        index_path.pop();
    }
}
