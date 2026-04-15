use crate::app::*;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    #[allow(clippy::too_many_lines)]
    pub(super) fn dispatch_action(&mut self, ctx: &egui::Context, action: AppAction) {
        match action {
            AppAction::PickOpenWorkspace => if let Some(path) = crate::shell_ui::ShellUiOps::open_folder_dialog() { self.handle_open_explorer(path); }
            AppAction::OpenWorkspace(p) => self.handle_open_explorer(p),
            AppAction::RefreshExplorer => self.handle_refresh_explorer(),
            AppAction::CreateFsNode { parent_dir, is_dir, target_path } => self.handle_action_create_fs_node(parent_dir, is_dir, target_path),
            AppAction::RenameFsNode { target_path, new_path } => self.handle_action_rename_fs_node(target_path, new_path),
            AppAction::DeleteFsNode { target_path } => self.handle_action_delete_fs_node(target_path),
            AppAction::SelectDocument(p) => self.handle_action_select_document(p),
            AppAction::SelectDocumentAndJump { path, line, byte_range: _ } => self.handle_action_select_and_jump(path, line),
            AppAction::OpenMultipleDocuments(paths) => self.handle_action_open_multiple(paths),
            AppAction::RemoveWorkspace(path) => self.handle_remove_explorer(path),
            AppAction::RemoveWorkspaceHistory(path) => self.handle_remove_workspace_history(path),
            AppAction::ShowStatusMessage(msg, status_type) => {
                self.state.layout.status_message = Some((msg, status_type))
            }
            AppAction::CloseWorkspace => {
                self.save_workspace_state();
                self.state.workspace.data = None;
                self.state.document.open_documents.clear();
                self.state.document.active_doc_idx = None;
                self.state.document.tab_groups.clear();
                self.state.document.tab_view_modes.clear();
                self.state.document.tab_split_states.clear();
                self.state.document.recently_closed_tabs.clear();
                self.state.search.filter_cache = None;
                self.state.layout.status_message = Some((
                    crate::i18n::I18nOps::get().status.closed_workspace.clone(),
                    crate::app_state::StatusType::Success,
                ));
            }
            AppAction::CloseDocument(idx) => self.handle_action_close_document(idx),
            AppAction::CloseActiveDocument => {
                if let Some(idx) = self.state.document.active_doc_idx {
                    self.handle_action_close_document(idx);
                }
            }
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
                self.state.config.settings.settings_mut().language = lang.clone();
                if !self.state.config.try_save_settings() {
                    tracing::warn!("Failed to save language setting");
                }
                /* WHY: Synchronize demo content localization if the demo is open */
                self.handle_action_switch_demo_lang(&lang);
            }
            AppAction::ToggleSettings => self.state.layout.show_settings ^= true,
            AppAction::ToggleExportPanel => {
                if !self.state.layout.show_export_panel {
                    self.state.layout.show_story_panel = false;
                    self.state.layout.show_tools_panel = false;
                    self.state.layout.show_toc = false;
                }
                self.state.layout.show_export_panel ^= true;
            }
            AppAction::ToggleStoryPanel => {
                if !self.state.layout.show_story_panel {
                    self.state.layout.show_export_panel = false;
                    self.state.layout.show_tools_panel = false;
                    self.state.layout.show_toc = false;
                }
                self.state.layout.show_story_panel ^= true;
            }
            AppAction::ToggleToolsPanel => {
                if !self.state.layout.show_tools_panel {
                    self.state.layout.show_export_panel = false;
                    self.state.layout.show_story_panel = false;
                    self.state.layout.show_toc = false;
                }
                self.state.layout.show_tools_panel ^= true;
            }
            AppAction::ToggleAbout => self.show_about = !self.show_about,
            AppAction::ToggleToc => {
                if !self.state.layout.show_toc {
                    self.state.layout.show_export_panel = false;
                    self.state.layout.show_story_panel = false;
                    self.state.layout.show_tools_panel = false;
                }
                self.state.layout.show_toc ^= true;
            }
            AppAction::ToggleWorkspacePanel => {
                let current = self.state.layout.show_workspace_panel;
                self.state.layout.show_workspace_panel = !current;
                if !current {
                    /* WHY: Reload from disk to show the latest persisted workspace list */
                    self.state.global_workspace.reload();
                }
            }
            AppAction::ToggleExplorer => {
                let current = self.state.layout.show_explorer;
                self.state.layout.show_explorer = !current;
                if !current {
                    /* WHY: Reload from disk to show the latest history in empty workspace view */
                    self.state.global_workspace.reload();
                }
            }
            AppAction::ToggleHistoryPanel => {
                let current = self.state.layout.show_history_panel;
                self.state.layout.show_history_panel = !current;
                if !current {
                    /* WHY: Reload from disk to show the latest history list */
                    self.state.global_workspace.reload();
                }
            }
            AppAction::ToggleSearchModal => {
                self.state.layout.show_search_modal = !self.state.layout.show_search_modal;
            }
            AppAction::ToggleCommandPalette => self.state.command_palette.toggle(),
            AppAction::ToggleRailPopup(popup) => {
                if self.state.layout.active_rail_popup == Some(popup) {
                    self.state.layout.active_rail_popup = None;
                } else {
                    self.state.layout.active_rail_popup = Some(popup);
                    /* WHY: Ensure the sidebar explorer remains visible whenever an activity rail panel is opened. */
                    self.state.layout.show_explorer = true;
                }
            }
            AppAction::ToggleSlideshow => self.handle_action_toggle_slideshow(ctx),
            AppAction::ToggleSlideshowHoverHighlight => self.state.layout.slideshow_hover_highlight ^= true,
            AppAction::ToggleSlideshowShowDiagramControls => self.state.layout.slideshow_show_diagram_controls ^= true,
            AppAction::OpenDocSearch => {
                self.state.search.doc_search_open = true;
                ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("search_newly_opened"), true));
            }
            AppAction::ToggleDocSearch => {
                if !self.state.search.doc_search_open {
                    self.state.search.doc_search_open = true;
                    ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("search_newly_opened"), true));
                    self.trigger_action(AppAction::DocSearchQueryChanged);
                } else {
                    self.state.search.doc_search_open = false;
                    self.state.search.doc_search_matches.clear();
                }
            }
            AppAction::DocSearchQueryChanged => self.handle_action_doc_search_changed(),
            AppAction::DocSearchNext => self.handle_action_doc_search_next(ctx),
            AppAction::DocSearchPrev => self.handle_action_doc_search_prev(ctx),
            AppAction::ToggleProblemsPanel => self.state.diagnostics.is_panel_open = !self.state.diagnostics.is_panel_open,
            AppAction::RefreshDiagnostics => self.handle_action_refresh_diagnostics(),
            AppAction::ToggleExplorerFilter => {
                let current = self.state.search.filter_enabled;
                self.state.search.filter_enabled = !current;
                if !current { ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("filter_newly_enabled"), true)); }
            }
            other => self.dispatch_secondary(ctx, other),
        }
    }
}
