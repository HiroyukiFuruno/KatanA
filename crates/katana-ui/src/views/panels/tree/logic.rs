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
                    let is_hidden_dir = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|s| s.starts_with('.'));

                    if is_hidden_dir {
                        continue;
                    }

                    if Self::gather_visible_paths(children, regex, is_negated, ws_root, visible) {
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
