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
    ) {
        if search.filter_enabled && !search.filter_query.is_empty() {
            let is_negated = search.filter_query.starts_with('!');
            let query_str = if is_negated {
                &search.filter_query[1..]
            } else {
                &search.filter_query
            };

            match regex::RegexBuilder::new(query_str)
                .case_insensitive(true)
                .build()
            {
                Ok(regex) => {
                    if search.filter_cache.as_ref().map(|(q, _)| q) != Some(&search.filter_query) {
                        let mut visible = HashSet::new();
                        crate::views::panels::tree::TreeLogicOps::gather_visible_paths(
                            entries,
                            &regex,
                            is_negated,
                            ws_root,
                            &mut visible,
                        );
                        search.filter_cache = Some((search.filter_query.clone(), visible));
                    }
                }
                Err(_) => {
                    search.filter_cache = None;
                }
            }
        } else {
            search.filter_cache = None;
        }
    }
}
