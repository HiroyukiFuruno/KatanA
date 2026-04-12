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
        if search.filter_enabled && !search.filter.query.is_empty() {
            let is_negated = search.filter.query.starts_with('!');
            let query_str = if is_negated {
                &search.filter.query[1..]
            } else {
                &search.filter.query
            };

            let (pattern, case_insensitive) = if search.filter.use_regex {
                let p = if search.filter.match_word {
                    format!(r"\b{}\b", query_str)
                } else {
                    query_str.to_string()
                };
                (p, !search.filter.match_case)
            } else {
                let p = regex::escape(query_str);
                let p = if search.filter.match_word {
                    format!(r"\b{}\b", p)
                } else {
                    p
                };
                (p, !search.filter.match_case)
            };

            match regex::RegexBuilder::new(&pattern)
                .case_insensitive(case_insensitive)
                .build()
            {
                Ok(regex) => {
                    if search.filter_cache.as_ref().map(|(q, _)| q) != Some(&search.filter) {
                        let mut visible = HashSet::new();
                        crate::views::panels::tree::TreeLogicOps::gather_visible_paths(
                            entries,
                            &regex,
                            is_negated,
                            ws_root,
                            &mut visible,
                        );
                        search.filter_cache = Some((search.filter.clone(), visible));
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
