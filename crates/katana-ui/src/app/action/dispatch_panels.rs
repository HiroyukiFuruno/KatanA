use crate::shell::*;
use crate::views::panels::preview::{PreviewMenu, PreviewMenuAvailability};

#[derive(Clone, Copy)]
enum PreviewPanel {
    Export,
    Story,
    Tools,
    Toc,
}

impl PreviewPanel {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "export" => Some(Self::Export),
            "story" => Some(Self::Story),
            "tools" => Some(Self::Tools),
            "toc" => Some(Self::Toc),
            _ => None,
        }
    }

    const fn menu(self) -> PreviewMenu {
        match self {
            Self::Export => PreviewMenu::Export,
            Self::Story => PreviewMenu::Story,
            Self::Tools => PreviewMenu::Tools,
            Self::Toc => PreviewMenu::Toc,
        }
    }

    fn is_open(self, layout: &crate::app_state::LayoutState) -> bool {
        match self {
            Self::Export => layout.show_export_panel,
            Self::Story => layout.show_story_panel,
            Self::Tools => layout.show_tools_panel,
            Self::Toc => layout.show_toc,
        }
    }

    fn set_open(self, layout: &mut crate::app_state::LayoutState, open: bool) {
        match self {
            Self::Export => layout.show_export_panel = open,
            Self::Story => layout.show_story_panel = open,
            Self::Tools => layout.show_tools_panel = open,
            Self::Toc => layout.show_toc = open,
        }
    }
}

impl KatanaApp {
    pub(super) fn handle_toggle_reload_panel(&mut self, panel: &str) {
        /* WHY: These panels need disk reload when opened to reflect latest state. */
        let flag = match panel {
            "workspace" => &mut self.state.layout.show_workspace_panel,
            "explorer" => &mut self.state.layout.show_explorer,
            "history" => &mut self.state.layout.show_history_panel,
            _ => return,
        };
        let was_open = *flag;
        *flag = !was_open;
        if !was_open {
            self.state.global_workspace.reload();
        }
    }

    pub(super) fn handle_toggle_panel(&mut self, panel: &str) {
        let Some(target) = PreviewPanel::from_name(panel) else {
            return;
        };
        if !PreviewMenuAvailability::for_path(
            self.state
                .active_document()
                .map(|document| document.path.as_path()),
            target.menu(),
        ) {
            target.set_open(&mut self.state.layout, false);
            return;
        }
        /* WHY: Close siblings before opening a new side panel. */
        let should_open = !target.is_open(&self.state.layout);
        if should_open {
            self.state.layout.show_export_panel = false;
            self.state.layout.show_story_panel = false;
            self.state.layout.show_tools_panel = false;
            self.state.layout.show_toc = false;
        }
        target.set_open(&mut self.state.layout, should_open);
    }
}
