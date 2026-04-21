use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
    CommandPaletteResultKind,
};
use katana_core::workspace::Workspace;

pub const MAX_FILE_RESULTS: usize = 50;
pub const BASE_CONTENT_SCORE: f32 = 0.6;
pub const FILE_PRIMARY_MATCH_SCORE: f32 = 0.9;
pub const FILE_SECONDARY_MATCH_SCORE: f32 = 0.5;

use crate::state::command_inventory::CommandInventory;

/// Provides common application actions from the shared CommandInventory.
pub struct AppCommandProvider;

impl CommandPaletteProvider for AppCommandProvider {
    fn name(&self) -> &'static str {
        "Commands"
    }

    fn search(
        &self,
        query: &str,
        _workspace: Option<&Workspace>,
        os_bindings: Option<&std::collections::HashMap<String, String>>,
    ) -> Vec<CommandPaletteResult> {
        let commands = CommandInventory::all();
        let msgs = &crate::i18n::I18nOps::get().search;

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let cmd_type = &msgs.command_type_action;

        /* WHY: For now we ignore the `is_available` to show all in palette, or we could filter. */
        /* WHY: Often palette shows commands but disables them, but here we just show all. */
        const COMMAND_BASE_SCORE: f32 = 0.8;
        const EMPTY_QUERY_PENALTY: f32 = 0.2;

        for cmd in commands {
            let label = (cmd.label)();
            let base_score = COMMAND_BASE_SCORE;

            let shortcut = os_bindings
                .and_then(|b| b.get(cmd.id).cloned())
                .or_else(|| cmd.default_shortcuts.first().map(|s| s.to_string()));

            let is_empty = query_lower.is_empty();
            let label_lower = label.to_lowercase();
            if is_empty || label_lower.contains(&query_lower) {
                let score = if is_empty {
                    base_score - EMPTY_QUERY_PENALTY
                } else {
                    base_score + label_lower.starts_with(&query_lower) as u32 as f32
                };
                let kind = if is_empty {
                    CommandPaletteResultKind::RecentOrCommon
                } else {
                    CommandPaletteResultKind::Action
                };
                results.push(CommandPaletteResult {
                    id: cmd.id.to_string(),
                    label,
                    secondary_label: Some(format!("{} - {}", cmd.group.localized_name(), cmd_type)),
                    shortcut,
                    score,
                    kind,
                    execute_payload: CommandPaletteExecutePayload::DispatchAppAction(
                        cmd.action.clone(),
                    ),
                });
            }
        }
        results
    }
}

/// Provides file results from the workspace.
pub struct WorkspaceFileProvider;

impl CommandPaletteProvider for WorkspaceFileProvider {
    fn name(&self) -> &'static str {
        "Files"
    }

    fn search(
        &self,
        query: &str,
        workspace: Option<&Workspace>,
        _os_bindings: Option<&std::collections::HashMap<String, String>>,
    ) -> Vec<CommandPaletteResult> {
        let mut results = Vec::new();
        if query.is_empty() {
            return results;
        }

        let query_lower = query.to_lowercase();

        if let Some(ws) = workspace {
            let mut file_matches: Vec<std::path::PathBuf> = Vec::new();
            crate::shell_logic::ShellLogicOps::collect_matches(
                &ws.tree,
                &query_lower,
                &[],
                &[],
                &ws.root,
                false, // match_case
                false, // match_word
                false, // use_regex
                &mut file_matches,
            );

            for file_path in file_matches.into_iter().take(MAX_FILE_RESULTS) {
                let rel_path = crate::shell_logic::ShellLogicOps::relative_full_path(
                    &file_path,
                    Some(&ws.root),
                );

                /* WHY: Keep the file name as the primary label */
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| rel_path.clone());

                let score = if file_name.to_lowercase().contains(&query_lower) {
                    FILE_PRIMARY_MATCH_SCORE
                } else {
                    FILE_SECONDARY_MATCH_SCORE
                };

                results.push(CommandPaletteResult {
                    id: format!("file_{}", rel_path),
                    label: file_name,
                    secondary_label: Some(rel_path.clone()),
                    shortcut: None,
                    score,
                    kind: CommandPaletteResultKind::File,
                    execute_payload: CommandPaletteExecutePayload::OpenFile(file_path),
                });
            }
        }

        results
    }
}

pub mod markdown_content;
pub use markdown_content::MarkdownContentProvider;
