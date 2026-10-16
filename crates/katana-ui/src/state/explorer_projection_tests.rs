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

    assert!(cache.refresh(&workspace, 1, &expanded, None, false));
    assert_eq!(cache.rows().len(), LARGE_WORKSPACE_FILE_COUNT + 1);
    assert!(!cache.refresh(&workspace, 1, &expanded, None, false));
    assert_eq!(cache.rebuild_count(), 1);
    let steady_bytes = cache.estimated_heap_bytes();
    assert!(steady_bytes > 0);
    assert!(!cache.refresh(&workspace, 1, &expanded, None, false));
    assert_eq!(cache.estimated_heap_bytes(), steady_bytes);

    let rescanned = large_workspace();
    assert!(cache.refresh(&rescanned, 2, &expanded, None, false));
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
    cache.refresh(&workspace, 1, &expanded, None, false);

    let entry = cache.rows()[1].resolve(&workspace.tree).unwrap();
    assert_eq!(entry.path(), Path::new("/workspace/docs/file-0.pdf"));
}

#[test]
fn workspace_state_revision_invalidates_projection_across_refresh_close_and_reopen() {
    let mut state = crate::state::WorkspaceState::new();
    let mut search = crate::state::SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "report".to_string();
    search.file_search.query = "draft".to_string();
    let workspace = large_workspace();
    state.set_data(Some(workspace));

    let expanded = HashSet::from([PathBuf::from("/workspace/docs")]);
    let mut cache = ExplorerProjectionCache::default();
    assert!(cache.refresh(
        state.data.as_ref().unwrap(),
        state.revision(),
        &expanded,
        None,
        false,
    ));
    assert!(!cache.refresh(
        state.data.as_ref().unwrap(),
        state.revision(),
        &expanded,
        None,
        false,
    ));

    let first_revision = state.revision();
    state.set_data(Some(large_workspace()));
    assert!(state.revision() > first_revision);
    assert!(cache.refresh(
        state.data.as_ref().unwrap(),
        state.revision(),
        &expanded,
        None,
        false,
    ));

    state.set_data(Some(Workspace::new(
        "/other-workspace",
        vec![TreeEntry::File {
            path: PathBuf::from("/other-workspace/readme.md"),
        }],
    )));
    assert!(cache.refresh(
        state.data.as_ref().unwrap(),
        state.revision(),
        &expanded,
        None,
        false,
    ));
    assert_eq!(cache.rows().len(), 1);

    let refresh_revision = state.revision();
    state.set_data(None);
    assert!(state.revision() > refresh_revision);
    let closed_revision = state.revision();
    state.set_data(Some(large_workspace()));
    assert!(state.revision() > closed_revision);
    assert!(cache.refresh(
        state.data.as_ref().unwrap(),
        state.revision(),
        &expanded,
        None,
        false,
    ));

    assert_eq!(expanded, HashSet::from([PathBuf::from("/workspace/docs")]));
    assert_eq!(search.filter.query, "report");
    assert_eq!(search.file_search.query, "draft");
}

#[test]
fn workspace_revision_invalidates_filter_cache_after_same_root_refresh() {
    let root = PathBuf::from("/workspace");
    let matching_path = root.join("report.md");
    let mut state = crate::state::WorkspaceState::new();
    let mut search = crate::state::SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "report".to_string();

    state.set_data(Some(Workspace::new(
        root.clone(),
        vec![TreeEntry::File {
            path: matching_path.clone(),
        }],
    )));
    let workspace = state.data.as_ref().unwrap();
    crate::views::panels::explorer::ExplorerLogicOps::update_search_filter_cache(
        &mut search,
        &workspace.root,
        &workspace.tree,
        state.revision(),
    );
    assert!(
        search
            .filter_cache
            .as_ref()
            .unwrap()
            .1
            .contains(&matching_path)
    );

    state.set_data(Some(Workspace::new(
        root.clone(),
        vec![TreeEntry::File {
            path: root.join("notes.md"),
        }],
    )));
    let workspace = state.data.as_ref().unwrap();
    crate::views::panels::explorer::ExplorerLogicOps::update_search_filter_cache(
        &mut search,
        &workspace.root,
        &workspace.tree,
        state.revision(),
    );
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.is_empty());
    assert_eq!(search.filter.query, "report");
    assert_eq!(
        search.filter_cache_workspace_revision,
        Some(state.revision())
    );
}
