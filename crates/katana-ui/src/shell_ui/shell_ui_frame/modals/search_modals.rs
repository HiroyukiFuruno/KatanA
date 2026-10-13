/* WHY: Isolated search and command palette orchestration for better modularity. */

use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(crate) fn show_search_and_palette_modals(&mut self, ctx: &egui::Context) {
        if self.state.command_palette.is_open {
            let providers: Vec<Box<dyn crate::state::command_palette::CommandPaletteProvider>> = vec![
                Box::new(crate::state::command_palette_providers::AppCommandProvider),
                Box::new(crate::state::command_palette_providers::WorkspaceFileProvider),
                Box::new(crate::state::command_palette_providers::MarkdownContentProvider),
            ];
            crate::views::modals::command_palette::CommandPaletteModal::new(
                &mut self.state.command_palette,
                self.state.workspace.data.as_ref(),
                &mut self.pending_action,
                &providers,
            )
            .show(ctx);
        }

        if self.state.layout.show_search_modal {
            let mut is_open = true;
            crate::views::modals::search::SearchModal::new(
                &mut self.state.search,
                self.state.workspace.data.as_ref(),
                &mut is_open,
                &mut self.pending_action,
            )
            .show(ctx);
            if !is_open && matches!(self.pending_action, AppAction::None) {
                self.pending_action = AppAction::ToggleSearchModal;
            }

            let recent = self.state.search.md_history.recent_terms.clone();
            let saved = self
                .state
                .config
                .settings
                .settings()
                .search
                .recent_md_queries
                .clone();
            if recent != saved {
                self.state
                    .config
                    .settings
                    .settings_mut()
                    .search
                    .recent_md_queries = recent;
                if !self.state.config.try_save_settings() {
                    tracing::warn!("Failed to save search history settings");
                }
            }
        }
    }
}
