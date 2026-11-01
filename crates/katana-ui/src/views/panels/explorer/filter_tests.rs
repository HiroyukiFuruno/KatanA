use super::types::ExplorerLogicOps;
use crate::app_state::SearchState;
use katana_core::workspace::TreeEntry;
use std::path::Path;

#[test]
fn regex_filter_matches_relative_paths_and_preserves_ancestors() {
    let root = Path::new("/workspace");
    let dir = root.join("images");
    let png = dir.join("icon.png");
    let text = dir.join("readme.txt");
    let entries = vec![TreeEntry::Directory {
        path: dir.clone(),
        children: vec![
            TreeEntry::File { path: png.clone() },
            TreeEntry::File { path: text.clone() },
        ],
    }];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = r".*\.png$".into();
    search.filter.use_regex = true;

    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&dir));
    assert!(visible.contains(&png));
    assert!(!visible.contains(&text));

    search.filter.use_regex = false;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(!search.filter_cache.as_ref().unwrap().1.contains(&png));
}

#[test]
fn invalid_regex_clears_existing_cache() {
    let root = Path::new("/workspace");
    let entries = vec![TreeEntry::File {
        path: root.join("a.png"),
    }];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "png".into();
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(search.filter_cache.is_some());

    search.filter.query = "[".into();
    search.filter.use_regex = true;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(search.filter_cache.is_none());
    assert!(search.filter_cache_workspace_revision.is_none());
}

#[test]
fn filter_mode_changes_refresh_cached_results() {
    let root = Path::new("/workspace");
    let exact = root.join("Image.png");
    let longer = root.join("ImageFile.png");
    let entries = vec![
        TreeEntry::File {
            path: exact.clone(),
        },
        TreeEntry::File {
            path: longer.clone(),
        },
    ];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "image".into();
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert_eq!(search.filter_cache.as_ref().unwrap().1.len(), 2);

    search.filter.match_case = true;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(search.filter_cache.as_ref().unwrap().1.is_empty());

    search.filter.match_case = false;
    search.filter.match_word = true;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&exact));
    assert!(!visible.contains(&longer));

    search.filter.query = "!image".into();
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(!visible.contains(&exact));
    assert!(visible.contains(&longer));
}
