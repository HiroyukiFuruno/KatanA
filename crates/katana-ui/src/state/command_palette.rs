use crate::app_state::AppAction;
use std::path::PathBuf;

/// The type of a command palette result, determining its origin and purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteResultKind {
    /// A generic application action.
    Action,
    /// A file in the workspace.
    File,
    /// A specific string or section inside a Markdown document.
    MarkdownContent,
    /// A recently or commonly used entry shown when the query is empty.
    RecentOrCommon,
}

/// The payload for executing a chosen result from the palette.
#[derive(Debug, Clone)]
pub enum CommandPaletteExecutePayload {
    /// Dispatch a predefined AppAction (e.g., ToggleSettings, RefreshWorkspace).
    DispatchAppAction(AppAction),
    /// Open a file in the workspace.
    OpenFile(PathBuf),
    /// Navigate to a specific location in a Markdown file.
    NavigateToContent {
        path: PathBuf,
        line: usize,
        byte_range: std::ops::Range<usize>,
    },
}

impl PartialEq for CommandPaletteExecutePayload {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DispatchAppAction(l), Self::DispatchAppAction(r)) => l == r,
            (Self::OpenFile(l), Self::OpenFile(r)) => l == r,
            (
                Self::NavigateToContent {
                    path: lp,
                    line: ll,
                    byte_range: lb,
                },
                Self::NavigateToContent {
                    path: rp,
                    line: rl,
                    byte_range: rb,
                },
            ) => lp == rp && ll == rl && lb == rb,
            _ => false,
        }
    }
}

/// A uniform result entry rendered in the command palette.
#[derive(Debug, Clone)]
pub struct CommandPaletteResult {
    /// Unique identifier for this result (useful for tracking selection).
    pub id: String,
    /// The primary text shown to the user (e.g., "Toggle Workspace" or "tasks.md").
    pub label: String,
    /// The secondary text providing context (e.g., "openspec/changes/...").
    pub secondary_label: Option<String>,
    /// Optional shortcut string (e.g. "cmd+s") to display next to the label.
    pub shortcut: Option<String>,
    /// Ranking score, higher is better.
    pub score: f32,
    /// The kind of result, used for rendering icons or colors.
    pub kind: CommandPaletteResultKind,
    /// What happens when the user selects and confirms this result.
    pub execute_payload: CommandPaletteExecutePayload,
}

impl PartialEq for CommandPaletteResult {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// A contract for entities that can supply results to the command palette.
pub trait CommandPaletteProvider {
    /// Returns the friendly name of this provider.
    fn name(&self) -> &'static str;

    /// Queries the provider for results matching the query string.
    /// If `query` is empty, the provider may return recent or common entries.
    fn search(
        &self,
        query: &str,
        workspace: Option<&katana_core::workspace::Workspace>,
        os_bindings: Option<&std::collections::HashMap<String, String>>,
    ) -> Vec<CommandPaletteResult>;
}

/// The state of the Command Palette UI and session.
#[derive(Debug, Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub current_query: String,
    pub selected_index: usize,
    pub results: Vec<CommandPaletteResult>,
    pub request_cursor_eof: bool,
}

impl CommandPaletteState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            current_query: String::new(),
            selected_index: 0,
            results: Vec::new(),
            request_cursor_eof: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.current_query.clear();
            self.selected_index = 0;
            self.results.clear();
            self.request_cursor_eof = true;
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn update_results(&mut self, results: Vec<CommandPaletteResult>) {
        self.results = results;
        if self.selected_index >= self.results.len() {
            self.selected_index = self.results.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.results.is_empty() {
            self.selected_index = self.results.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.results.is_empty() {
            if self.selected_index < self.results.len() - 1 {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }
}
