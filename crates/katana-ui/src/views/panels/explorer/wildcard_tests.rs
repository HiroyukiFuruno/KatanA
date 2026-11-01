use super::types::ExplorerLogicOps;
use crate::app_state::SearchState;
use katana_core::workspace::TreeEntry;
use std::path::{Path, PathBuf};

#[test]
fn wildcard_filter_matches_png_basenames_and_preserves_parent_directories() {
    let root = Path::new("/workspace");
    let root_png = root.join("root.png");
    let images = root.join("images");
    let nested_png = images.join("nested.PNG");
    let not_png = images.join("notes.txt");
    let png_backup = images.join("nested.png.bak");
    let copied_assets = root.join("assets (copy)");
    let copied_png = copied_assets.join("icon.png");
    let hyphenated_assets = root.join("assets-");
    let hyphenated_png = hyphenated_assets.join("icon.png");
    let entries = vec![
        TreeEntry::File {
            path: root_png.clone(),
        },
        TreeEntry::Directory {
            path: images.clone(),
            children: vec![
                TreeEntry::File {
                    path: nested_png.clone(),
                },
                TreeEntry::File { path: not_png },
                TreeEntry::File { path: png_backup },
            ],
        },
        TreeEntry::Directory {
            path: copied_assets.clone(),
            children: vec![TreeEntry::File {
                path: copied_png.clone(),
            }],
        },
        TreeEntry::Directory {
            path: hyphenated_assets.clone(),
            children: vec![TreeEntry::File {
                path: hyphenated_png.clone(),
            }],
        },
    ];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "*.png".into();

    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);

    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&root_png));
    assert!(visible.contains(&nested_png));
    assert!(visible.contains(&images));
    assert_eq!(visible.len(), 7);

    search.filter.match_word = true;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&root_png));
    assert!(visible.contains(&nested_png));
    assert!(visible.contains(&copied_assets));
    assert!(visible.contains(&copied_png));
    assert!(visible.contains(&hyphenated_assets));
    assert!(visible.contains(&hyphenated_png));
}

#[test]
fn wildcard_path_patterns_match_only_the_requested_path_segment() {
    let root = Path::new("/workspace");
    let assets = root.join("assets");
    let direct_png = assets.join("icon.png");
    let nested_png = assets.join("icons").join("nested.png");
    let entries = vec![TreeEntry::Directory {
        path: assets.clone(),
        children: vec![
            TreeEntry::File {
                path: direct_png.clone(),
            },
            TreeEntry::Directory {
                path: assets.join("icons"),
                children: vec![TreeEntry::File {
                    path: nested_png.clone(),
                }],
            },
        ],
    }];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "assets/*.png".into();

    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);

    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&assets));
    assert!(visible.contains(&direct_png));
    assert!(!visible.contains(&nested_png));
}

#[test]
fn wildcard_filter_respects_negation_case_and_literal_special_characters() {
    let root = Path::new("/workspace");
    let lower_png = root.join("logo.png");
    let upper_png = root.join("logo.PNG");
    let bracketed = root.join("name[1].txt");
    let plain = root.join("name1.txt");
    let entries = vec![
        TreeEntry::File {
            path: lower_png.clone(),
        },
        TreeEntry::File {
            path: upper_png.clone(),
        },
        TreeEntry::File {
            path: bracketed.clone(),
        },
        TreeEntry::File {
            path: plain.clone(),
        },
    ];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "!*.png".into();

    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(!visible.contains(&lower_png));
    assert!(!visible.contains(&upper_png));
    assert!(visible.contains(&bracketed));

    search.filter.query = "*.png".into();
    search.filter.match_case = true;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&lower_png));
    assert!(!visible.contains(&upper_png));

    search.filter.query = "[".into();
    search.filter.match_case = false;
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&bracketed));
    assert!(!visible.contains(&plain));
}

#[cfg(unix)]
#[test]
fn unix_wildcards_treat_backslash_as_a_filename_character() {
    let root = Path::new("");
    let filename = PathBuf::from(r"a\b.png");
    let entries = vec![TreeEntry::File {
        path: filename.clone(),
    }];
    let mut search = SearchState::new();
    search.filter_enabled = true;

    search.filter.query = "a*.png".into();
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(search.filter_cache.as_ref().unwrap().1.contains(&filename));

    search.filter.query = "b*.png".into();
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(!search.filter_cache.as_ref().unwrap().1.contains(&filename));

    search.filter.query = "assets/".into();
    ExplorerLogicOps::update_search_filter_cache(&mut search, root, &entries, 1);
    assert!(!search.filter_cache.as_ref().unwrap().1.contains(&filename));
}

#[cfg(windows)]
#[test]
fn wildcard_cache_matches_windows_joined_tree_paths() {
    let root = PathBuf::from(r"C:\workspace");
    let assets = root.join("assets");
    let direct = assets.join("icon.png");
    let nested = assets.join("nested").join("icon.png");
    let entries = vec![TreeEntry::Directory {
        path: assets.clone(),
        children: vec![
            TreeEntry::File {
                path: direct.clone(),
            },
            TreeEntry::Directory {
                path: assets.join("nested"),
                children: vec![TreeEntry::File {
                    path: nested.clone(),
                }],
            },
        ],
    }];
    let mut search = SearchState::new();
    search.filter_enabled = true;
    search.filter.query = "assets/*.png".into();

    ExplorerLogicOps::update_search_filter_cache(&mut search, &root, &entries, 1);
    let visible = &search.filter_cache.as_ref().unwrap().1;
    assert!(visible.contains(&assets));
    assert!(visible.contains(&direct));
    assert!(!visible.contains(&nested));
}
