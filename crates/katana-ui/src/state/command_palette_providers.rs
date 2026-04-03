use crate::app_state::AppAction;
use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
    CommandPaletteResultKind,
};
use katana_core::workspace::Workspace;

const MAX_FILE_RESULTS: usize = 50;
const BASE_CONTENT_SCORE: f32 = 0.6;
const FILE_PRIMARY_MATCH_SCORE: f32 = 0.9;
const FILE_SECONDARY_MATCH_SCORE: f32 = 0.5;

/// Provides common application actions.
pub struct AppCommandProvider;

impl CommandPaletteProvider for AppCommandProvider {
    fn name(&self) -> &'static str {
        "Commands"
    }

    fn search(&self, query: &str, _workspace: Option<&Workspace>) -> Vec<CommandPaletteResult> {
        let msgs = &crate::i18n::get().search;
        let commands = vec![
            (&msgs.command_settings, AppAction::ToggleSettings, 0.9),
            (&msgs.command_workspace, AppAction::ToggleWorkspace, 0.8),
            (&msgs.command_close_all, AppAction::CloseAllDocuments, 0.7),
            (&msgs.command_refresh, AppAction::RefreshWorkspace, 0.7),
            (&msgs.command_updates, AppAction::CheckForUpdates, 0.6),
            (&msgs.command_about, AppAction::ToggleAbout, 0.6),
        ];

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        let cmd_type = &msgs.command_type_action;

        for (label, action, base_score) in commands {
            if query_lower.is_empty() {
                // If empty query, we return all as recent/common
                results.push(CommandPaletteResult {
                    id: format!("cmd_{}", label),
                    label: label.to_string(),
                    secondary_label: Some(cmd_type.clone()),
                    score: base_score, // lower base score for empty query so history can be higher
                    kind: CommandPaletteResultKind::RecentOrCommon,
                    execute_payload: CommandPaletteExecutePayload::DispatchAppAction(action),
                });
            } else if label.to_lowercase().contains(&query_lower) {
                // Calculate basic score
                let score = if label.to_lowercase().starts_with(&query_lower) {
                    base_score + 1.0
                } else {
                    base_score
                };

                results.push(CommandPaletteResult {
                    id: format!("cmd_{}", label),
                    label: label.to_string(),
                    secondary_label: Some(cmd_type.clone()),
                    score,
                    kind: CommandPaletteResultKind::Action,
                    execute_payload: CommandPaletteExecutePayload::DispatchAppAction(action),
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
            let mut file_matches = Vec::new();
            crate::shell_logic::ShellLogicOps::collect_matches(
                &ws.tree,
                &query_lower,
                &[], // no include
                &[], // no exclude
                &ws.root,
                &mut file_matches,
            );

            for file_path in file_matches.into_iter().take(MAX_FILE_RESULTS) {
                let rel_path = crate::shell_logic::ShellLogicOps::relative_full_path(
                    &file_path,
                    Some(&ws.root),
                );

                // Keep the file name as the primary label
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
            let matches = katana_core::search::search_workspace(ws, query, MAX_FILE_RESULTS);
            for m in matches {
                let rel_path = crate::shell_logic::ShellLogicOps::relative_full_path(
                    &m.file_path,
                    Some(&ws.root),
                );

                let mut label = m.snippet.clone();
                if label.len() > 100 {
                    label.truncate(100);
                    label.push_str("...");
                }

                results.push(CommandPaletteResult {
                    id: format!("content_{}_{}", rel_path, m.line_number),
                    label: label.trim().to_string(),
                    secondary_label: Some(format!("{}:{}", rel_path, m.line_number + 1)),
                    score: BASE_CONTENT_SCORE, // slightly lower score than exact file matches
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
