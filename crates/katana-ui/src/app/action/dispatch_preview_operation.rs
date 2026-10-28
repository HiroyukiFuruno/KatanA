use crate::app_state::AppAction;
use crate::shell::KatanaApp;
use eframe::egui;

impl KatanaApp {
    pub(super) fn dispatch_preview_operation_action(
        &mut self,
        ctx: &egui::Context,
        action: &AppAction,
    ) -> bool {
        match action {
            AppAction::SetSplitDirection(direction) => {
                if crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                    &self.state,
                    crate::state::document::ViewMode::Split,
                ) {
                    self.state.set_active_split_direction(*direction);
                }
                true
            }
            AppAction::SetPaneOrder(order) => {
                if crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                    &self.state,
                    crate::state::document::ViewMode::Split,
                ) {
                    self.state.set_active_pane_order(*order);
                }
                true
            }
            AppAction::SetViewMode(mode) => {
                self.refresh_document_operation_runtime();
                if crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                    &self.state,
                    *mode,
                ) {
                    self.state.set_active_view_mode(*mode);
                }
                true
            }
            AppAction::ToggleScrollSync(is_on) => {
                if crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                    &self.state,
                    crate::state::document::ViewMode::Split,
                ) {
                    self.state.scroll.sync_override = Some(*is_on);
                }
                true
            }
            AppAction::OpenDocSearch => {
                self.open_document_search(ctx);
                true
            }
            AppAction::ToggleDocSearch => {
                if self.state.search.doc_search_open {
                    self.state.search.doc_search_open = false;
                    self.state.search.doc_search_matches.clear();
                } else if self.open_document_search(ctx) {
                    self.trigger_action(AppAction::DocSearchQueryChanged);
                }
                true
            }
            AppAction::ToggleSplitMode => {
                self.refresh_document_operation_runtime();
                if crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                    &self.state,
                    crate::state::document::ViewMode::Split,
                ) {
                    self.state
                        .set_active_view_mode(crate::state::document::ViewMode::Split);
                }
                true
            }
            AppAction::ToggleCodePreview => {
                self.refresh_document_operation_runtime();
                let next = match self.state.active_view_mode() {
                    crate::state::document::ViewMode::Split
                    | crate::state::document::ViewMode::CodeOnly => {
                        crate::state::document::ViewMode::PreviewOnly
                    }
                    crate::state::document::ViewMode::PreviewOnly => {
                        crate::state::document::ViewMode::CodeOnly
                    }
                };
                if crate::views::panels::preview::PreviewMenuAvailability::for_view_mode(
                    &self.state,
                    next,
                ) {
                    self.state.set_active_view_mode(next);
                }
                true
            }
            _ => false,
        }
    }

    fn open_document_search(&mut self, ctx: &egui::Context) -> bool {
        self.refresh_document_operation_runtime();
        if !crate::views::panels::preview::PreviewMenuAvailability::for_state(
            &self.state,
            crate::views::panels::preview::PreviewMenu::Search,
        ) {
            return false;
        }
        self.state.search.doc_search_open = true;
        ctx.memory_mut(|memory| {
            memory
                .data
                .insert_temp(egui::Id::new("search_newly_opened"), true)
        });
        true
    }
}
