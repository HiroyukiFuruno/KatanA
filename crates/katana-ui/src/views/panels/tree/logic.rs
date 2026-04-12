use super::types::*;

impl TreeLogicOps {
    pub fn gather_visible_paths(
        entries: &[katana_core::workspace::TreeEntry],
        regex: &regex::Regex,
        is_negated: bool,
        ws_root: &std::path::Path,
        visible: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> bool {
        let mut any_visible = false;
        for entry in entries {
            match entry {
                katana_core::workspace::TreeEntry::File { path } => {
                    let rel =
                        crate::shell_logic::ShellLogicOps::relative_full_path(path, Some(ws_root));
                    let is_match = regex.is_match(&rel);
                    let should_show = if is_negated { !is_match } else { is_match };

                    if should_show {
                        visible.insert(path.clone());
                        any_visible = true;
                    }
                }
                katana_core::workspace::TreeEntry::Directory { path, children } => {
                    let rel =
                        crate::shell_logic::ShellLogicOps::relative_full_path(path, Some(ws_root));

                    /* WHY: Hidden directories (e.g., .git) are included in the search if they match
                    the regex or contain any matching children. Regular filtering for hidden
                    folders when SEARCH IS OFF is handled in the non-filtered tree rendering. */
                    let is_match = regex.is_match(&rel);
                    /* WHY: Match the directory itself based on the regex and negation state. */
                    let should_show_self = if is_negated { !is_match } else { is_match };

                    /* WHY: Recursively check if any children are visible. */
                    let any_child_visible =
                        Self::gather_visible_paths(children, regex, is_negated, ws_root, visible);

                    /* WHY: Show the directory if it matches explicitly or if any of its children match. */
                    if any_child_visible || should_show_self {
                        visible.insert(path.clone());
                        any_visible = true;
                    }
                }
            }
        }
        any_visible
    }

    pub fn find_node_in_tree<'a>(
        entries: &'a [katana_core::workspace::TreeEntry],
        target: &std::path::Path,
    ) -> Option<&'a katana_core::workspace::TreeEntry> {
        for entry in entries {
            match entry {
                katana_core::workspace::TreeEntry::Directory { path, children } => {
                    if path == target {
                        return Some(entry);
                    }
                    if target.starts_with(path)
                        && let Some(found) = Self::find_node_in_tree(children, target)
                    {
                        return Some(found);
                    }
                }
                katana_core::workspace::TreeEntry::File { path } => {
                    if path == target {
                        return Some(entry);
                    }
                }
            }
        }
        None
    }
}
