use crate::state::command_palette::{
    CommandPaletteExecutePayload, CommandPaletteProvider, CommandPaletteResult,
    CommandPaletteResultKind,
};
use katana_core::workspace::Workspace;

/// Provides markdown content results.
pub struct MarkdownContentProvider;

impl CommandPaletteProvider for MarkdownContentProvider {
    fn name(&self) -> &'static str {
        "Content"
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

        if let Some(ws) = workspace {
            let matches = katana_core::search::WorkspaceSearchOps::search_workspace(
                ws,
                query,
                false, // match_case
                false, // match_word
                false, // use_regex
                super::MAX_FILE_RESULTS,
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
                    shortcut: None,
                    score: super::BASE_CONTENT_SCORE,
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
