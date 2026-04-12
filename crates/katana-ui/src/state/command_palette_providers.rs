use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
    CommandPaletteResultKind,
};
use katana_core::workspace::Workspace;

const MAX_FILE_RESULTS: usize = 50;
const BASE_CONTENT_SCORE: f32 = 0.6;
const FILE_PRIMARY_MATCH_SCORE: f32 = 0.9;
const FILE_SECONDARY_MATCH_SCORE: f32 = 0.5;

use crate::state::command_inventory::CommandInventory;

/// Provides common application actions from the shared CommandInventory.
pub struct AppCommandProvider;

impl CommandPaletteProvider for AppCommandProvider {
    fn name(&self) -> &'static str {
        "Commands"
    }

    fn search(&self, query: &str, _workspace: Option<&Workspace>) -> Vec<CommandPaletteResult> {
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

            if query_lower.is_empty() {
                /* WHY: If empty query, we return all as recent/common */
                results.push(CommandPaletteResult {
                    id: cmd.id.to_string(),
                    label: label.clone(),
                    secondary_label: Some(format!("{} - {}", cmd.group.localized_name(), cmd_type)),
                    /* WHY: lower base score for empty query so history can be higher */
                    score: base_score - EMPTY_QUERY_PENALTY,
                    kind: CommandPaletteResultKind::RecentOrCommon,
                    execute_payload: CommandPaletteExecutePayload::DispatchAppAction(
                        cmd.action.clone(),
                    ),
                });
            } else if label.to_lowercase().contains(&query_lower) {
                /* WHY: Calculate basic score */
                let bonus = label.to_lowercase().starts_with(&query_lower) as u32 as f32;
                let score = base_score + bonus;
                results.push(CommandPaletteResult {
                    id: cmd.id.to_string(),
                    label: label.clone(),
                    secondary_label: Some(format!("{} - {}", cmd.group.localized_name(), cmd_type)),
                    score,
                    kind: CommandPaletteResultKind::Action,
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

    fn search(&self, query: &str, workspace: Option<&Workspace>) -> Vec<CommandPaletteResult> {
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
                    score,
                    kind: CommandPaletteResultKind::File,
                    execute_payload: CommandPaletteExecutePayload::OpenFile(file_path),
                });
            }
        }

        results
    }
}

/// Provides markdown content results.
pub struct MarkdownContentProvider;

impl CommandPaletteProvider for MarkdownContentProvider {
    fn name(&self) -> &'static str {
        "Content"
    }

    fn search(&self, query: &str, workspace: Option<&Workspace>) -> Vec<CommandPaletteResult> {
        let mut results = Vec::new();
        if query.is_empty() {
            return results;
        }

        if let Some(ws) = workspace {
            let matches = katana_core::search::WorkspaceSearchOps::search_workspace(
                ws,
                query,
                false, // match_case
                false, // match_word
                false, // use_regex
                MAX_FILE_RESULTS,
            );
            for m in matches {
                let rel_path = crate::shell_logic::ShellLogicOps::relative_full_path(
                    &m.file_path,
                    Some(&ws.root),
                );

                let mut label = m.snippet.clone();
                if label.len() > 100 {
                    let end = label.floor_char_boundary(100);
                    label.truncate(end);
                    label.push_str("...");
                }

                results.push(CommandPaletteResult {
                    id: format!("content_{}_{}", rel_path, m.line_number),
                    label: label.trim().to_string(),
                    secondary_label: Some(format!("{}:{}", rel_path, m.line_number + 1)),
                    /* WHY: slightly lower score than exact file matches */
                    score: BASE_CONTENT_SCORE,
                    kind: CommandPaletteResultKind::MarkdownContent,
                    execute_payload: CommandPaletteExecutePayload::NavigateToContent {
                        path: m.file_path.clone(),
                        line: m.line_number,
                        byte_range: m.start_col..m.end_col,
                    },
                });
            }
        }

        results
    }
}
