use super::*;
use katana_core::workspace::TreeEntry;
use katana_core::workspace::Workspace;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const LARGE_WORKSPACE_FILE_COUNT: usize = 10_000;

fn large_workspace() -> Workspace {
    let children = (0..LARGE_WORKSPACE_FILE_COUNT)
        .map(|index| TreeEntry::File {
            path: PathBuf::from(format!("/workspace/docs/file-{index}.pdf")),
        })
        .collect();
    Workspace::new(
        "/workspace",
        vec![TreeEntry::Directory {
            path: PathBuf::from("/workspace/docs"),
            children,
        }],
    )
}

#[test]
fn projection_rebuilds_only_when_revision_or_view_state_changes() {
    let workspace = large_workspace();
    let mut expanded = HashSet::new();
    expanded.insert(PathBuf::from("/workspace/docs"));
    let mut cache = ExplorerProjectionCache::default();

    assert!(cache.refresh(&workspace, &expanded, None, false));
    assert_eq!(cache.rows().len(), LARGE_WORKSPACE_FILE_COUNT + 1);
    assert!(!cache.refresh(&workspace, &expanded, None, false));
    assert_eq!(cache.rebuild_count(), 1);
    let steady_bytes = cache.estimated_heap_bytes();
    assert!(steady_bytes > 0);
    assert!(!cache.refresh(&workspace, &expanded, None, false));
    assert_eq!(cache.estimated_heap_bytes(), steady_bytes);

    let rescanned = large_workspace();
    assert!(cache.refresh(&rescanned, &expanded, None, false));
    assert_eq!(cache.rebuild_count(), 2);
}

#[test]
fn viewport_limits_widget_rows_for_large_document_trees() {
    let range = ExplorerProjectionCache::visible_row_range(
        2200.0..2640.0,
        22.0,
        LARGE_WORKSPACE_FILE_COUNT + 1,
    );

    assert_eq!(range.start, 100);
    assert!(range.len() <= 22);
}

#[test]
fn projected_index_path_resolves_without_cloning_the_tree() {
    let workspace = large_workspace();
    let expanded = HashSet::from([PathBuf::from("/workspace/docs")]);
    let mut cache = ExplorerProjectionCache::default();
    cache.refresh(&workspace, &expanded, None, false);

    let entry = cache.rows()[1].resolve(&workspace.tree).unwrap();
    assert_eq!(entry.path(), Path::new("/workspace/docs/file-0.pdf"));
}
