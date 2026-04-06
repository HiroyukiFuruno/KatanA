use crate::app::*;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    #[allow(clippy::too_many_lines)]
    pub(super) fn dispatch_action(&mut self, ctx: &egui::Context, action: AppAction) {
        match action {
            AppAction::OpenWorkspace(p) => self.handle_open_workspace(p),
            AppAction::RefreshWorkspace => self.handle_refresh_workspace(),
            AppAction::CreateFsNode {
                parent_dir,
                is_dir,
                target_path,
            } => self.handle_action_create_fs_node(parent_dir, is_dir, target_path),
            AppAction::RenameFsNode {
                target_path,
                new_path,
            } => self.handle_action_rename_fs_node(target_path, new_path),
            AppAction::DeleteFsNode { target_path } => {
                self.handle_action_delete_fs_node(target_path)
            }
            AppAction::SelectDocument(p) => self.handle_action_select_document(p),
            AppAction::SelectDocumentAndJump {
                path,
                line,
                byte_range: _,
            } => self.handle_action_select_and_jump(path, line),
            AppAction::OpenMultipleDocuments(paths) => self.handle_action_open_multiple(paths),
            AppAction::RemoveWorkspace(path) => self.handle_remove_workspace(path),
            AppAction::CloseDocument(idx) => self.handle_action_close_document(idx),
            AppAction::ForceCloseDocument(idx) => {
                self.state.layout.pending_close_confirm = None;
                self.force_close_document(idx);
            }
            AppAction::UpdateBuffer(c) => self.handle_update_buffer(c),
            AppAction::ReplaceText { span, replacement } => {
                self.handle_replace_text(span, replacement)
            }
            AppAction::ToggleTaskList {
                global_index,
                new_state,
            } => self.handle_toggle_task_list(global_index, new_state),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => self.handle_action_refresh_diagrams(ctx),
            AppAction::RefreshDocument { is_manual } => {
                self.handle_action_refresh_document(ctx, is_manual)
            }
            AppAction::ChangeLanguage(lang) => {
                crate::i18n::I18nOps::set_language(&lang);
                crate::shell_ui::ShellUiOps::update_native_menu_strings_from_i18n();
                self.state.config.settings.settings_mut().language = lang;
                if !self.state.config.try_save_settings() {
                    tracing::warn!("Failed to save language setting");
                }
            }
            AppAction::ToggleSettings => {
                self.state.layout.show_settings = !self.state.layout.show_settings;
            }
            AppAction::ToggleAbout => self.show_about = !self.show_about,
            AppAction::ToggleToc => self.state.layout.show_toc = !self.state.layout.show_toc,
            AppAction::ToggleWorkspace => {
                self.state.layout.show_workspace = !self.state.layout.show_workspace;
            }
            AppAction::ToggleWorkspaceHistoryModal => {
                self.state.layout.show_workspace_history_modal =
                    !self.state.layout.show_workspace_history_modal;
            }
            AppAction::ToggleSearchModal => {
                self.state.layout.show_search_modal = !self.state.layout.show_search_modal;
            }
            AppAction::ToggleCommandPalette => self.state.command_palette.toggle(),
            AppAction::ToggleSlideshow => self.handle_action_toggle_slideshow(ctx),
            AppAction::OpenDocSearch => {
                self.state.search.doc_search_open = true;
                ctx.memory_mut(|m| {
                    m.data
                        .insert_temp(egui::Id::new("search_newly_opened"), true)
                });
            }
            AppAction::DocSearchQueryChanged => self.handle_action_doc_search_changed(),
            AppAction::DocSearchNext => self.handle_action_doc_search_next(),
            AppAction::DocSearchPrev => self.handle_action_doc_search_prev(),
            AppAction::ToggleProblemsPanel => {
                self.state.diagnostics.is_panel_open = !self.state.diagnostics.is_panel_open;
            }
            AppAction::RefreshDiagnostics => self.handle_action_refresh_diagnostics(),
            AppAction::ToggleWorkspaceFilter => {
                self.state.search.filter_enabled = !self.state.search.filter_enabled;
            }
            AppAction::SetSplitDirection(dir) => self.state.set_active_split_direction(dir),
            AppAction::SetPaneOrder(order) => self.state.set_active_pane_order(order),
            AppAction::SetViewMode(mode) => self.state.set_active_view_mode(mode),
            AppAction::ToggleScrollSync(is_on) => {
                self.state.scroll.sync_override = Some(is_on);
            }
            other => self.dispatch_secondary(ctx, other),
        }
    }
}
