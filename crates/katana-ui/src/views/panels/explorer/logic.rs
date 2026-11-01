use super::filter_pattern::FilterPattern;
use super::types::*;
use crate::app_state::{SearchState, WorkspaceState};
use katana_core::workspace::TreeEntry;
use std::collections::HashSet;
use std::path::Path;

impl ExplorerLogicOps {
    pub fn update_tree_expansion(workspace: &mut WorkspaceState) {
        if let Some(force) = workspace.force_tree_open {
            if let Some(ws) = &workspace.data {
                if force {
                    workspace
                        .expanded_directories
                        .extend(ws.collect_all_directory_paths());
                } else {
                    workspace.expanded_directories.clear();
                }
            }
            workspace.force_tree_open = None;
        }
    }

    pub fn update_search_filter_cache(
        search: &mut SearchState,
        ws_root: &Path,
        entries: &[TreeEntry],
        workspace_revision: u64,
    ) {
        if search.filter_enabled && !search.filter.query.is_empty() {
            match FilterPattern::compile(&search.filter) {
                Ok(pattern) => {
                    let cache_matches = search.filter_cache.as_ref().map(|(q, _)| q)
                        == Some(&search.filter)
                        && search.filter_cache_workspace_revision == Some(workspace_revision);
                    if !cache_matches {
                        let mut visible = HashSet::new();
                        crate::views::panels::tree::TreeLogicOps::gather_visible_paths(
                            entries,
                            &pattern.regex,
                            pattern.is_negated,
                            ws_root,
                            &mut visible,
                        );
                        search.filter_cache = Some((search.filter.clone(), visible));
                        search.filter_cache_workspace_revision = Some(workspace_revision);
                    }
                }
                Err(_) => {
                    search.filter_cache = None;
                    search.filter_cache_workspace_revision = None;
                }
            }
        } else {
            search.filter_cache = None;
            search.filter_cache_workspace_revision = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_filter_hides_html_file_when_query_does_not_match() {
        let root = std::path::PathBuf::from("/workspace");
        let index = root.join("index.html");
        let readme = root.join("README.md");
        let entries = vec![
            TreeEntry::File {
                path: index.clone(),
            },
            TreeEntry::File {
                path: readme.clone(),
            },
        ];
        let mut search = SearchState::new();
        search.filter_enabled = true;
        search.filter.query = "README".to_string();

        ExplorerLogicOps::update_search_filter_cache(&mut search, &root, &entries, 1);

        let Some((_, visible)) = search.filter_cache else {
            panic!("filter cache should be populated");
        };
        assert!(visible.contains(&readme));
        assert!(!visible.contains(&index));
    }
}
