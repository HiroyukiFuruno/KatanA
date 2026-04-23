use crate::app::*;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    #[allow(clippy::too_many_lines)]
    pub(super) fn dispatch_action(&mut self, ctx: &egui::Context, action: AppAction) {
        match action {
            AppAction::PickOpenWorkspace => {
                if crate::shell_ui::ShellUiOps::is_headless() {
                    self.pending_dialog_action = Some(AppAction::PickOpenWorkspace);
                    self.file_dialog.pick_directory();
                } else if let Some(path) = crate::shell_ui::ShellUiOps::open_folder_dialog() {
                    self.handle_open_explorer(path);
                } else {
                    self.pending_dialog_action = Some(AppAction::PickOpenWorkspace);
                    self.file_dialog.pick_directory();
                }
            }
            AppAction::OpenWorkspace(p) => self.handle_open_explorer(p),
            AppAction::RefreshExplorer => self.handle_refresh_explorer(),
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
            AppAction::RemoveWorkspace(path) => self.handle_remove_explorer(path),
            AppAction::RemoveWorkspaceHistory(path) => self.handle_remove_workspace_history(path),
            AppAction::ShowStatusMessage(msg, status_type) => {
                self.state.layout.status_message = Some((msg, status_type))
            }
            AppAction::CloseWorkspace => self.handle_action_close_workspace(),
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
            AppAction::ApplyLintFixes(fixes) => self.handle_apply_lint_fixes(fixes),
            AppAction::ToggleTaskList {
                global_index,
                new_state,
            } => self.handle_toggle_task_list(global_index, new_state),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => self.handle_action_refresh_diagrams(ctx),
            AppAction::RefreshDocument { is_manual } => {
                self.handle_action_refresh_document(ctx, is_manual)
            }
            AppAction::ChangeLanguage(lang) => self.handle_action_change_language(lang),
            AppAction::ToggleSettings => self.state.layout.show_settings ^= true,
            AppAction::ToggleExportPanel => self.handle_toggle_panel("export"),
            AppAction::ToggleStoryPanel => self.handle_toggle_panel("story"),
            AppAction::ToggleToolsPanel => self.handle_toggle_panel("tools"),
            AppAction::ToggleAbout => self.show_about = !self.show_about,
            AppAction::ToggleToc => self.handle_toggle_panel("toc"),
            AppAction::ToggleWorkspacePanel => self.handle_toggle_reload_panel("workspace"),
            AppAction::ToggleExplorer => self.handle_toggle_reload_panel("explorer"),
            AppAction::ToggleHistoryPanel => self.handle_toggle_reload_panel("history"),
            AppAction::ToggleSearchModal => self.state.layout.show_search_modal ^= true,
            AppAction::ToggleCommandPalette => self.state.command_palette.toggle(),
            AppAction::ToggleKatanaCommandPalette => {
                self.state.command_palette.toggle();
                if self.state.command_palette.is_open {
                    self.state.command_palette.current_query = ">".to_string();
                }
            }
            AppAction::ToggleRailPopup(popup) => {
                if self.state.layout.active_rail_popup == Some(popup) {
                    self.state.layout.active_rail_popup = None;
                } else {
                    self.state.layout.active_rail_popup = Some(popup);
                    /* WHY: Ensure the sidebar explorer remains visible when a rail panel opens. */
                    self.state.layout.show_explorer = true;
                }
            }
            AppAction::ToggleSlideshow => self.handle_action_toggle_slideshow(ctx),
            AppAction::ToggleSlideshowHoverHighlight => {
                self.state.layout.slideshow_hover_highlight ^= true
            }
            AppAction::ToggleSlideshowShowDiagramControls => {
                self.state.layout.slideshow_show_diagram_controls ^= true
            }
            other => self.dispatch_secondary(ctx, other),
        }
    }

    fn handle_toggle_reload_panel(&mut self, p: &str) {
        /* WHY: These panels need disk reload when opened to reflect latest state */
        let flag = match p {
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

    fn handle_action_change_language(&mut self, lang: String) {
        crate::i18n::I18nOps::set_language(&lang);
        crate::shell_ui::ShellUiOps::update_native_menu_strings_from_i18n();
        self.state.config.settings.settings_mut().language = lang.clone();
        if !self.state.config.try_save_settings() {
            tracing::warn!("Failed to save language setting");
        }
        /* WHY: Synchronize demo content localization if the demo is open */
        self.handle_action_switch_demo_lang(&lang);
    }

    fn handle_action_close_workspace(&mut self) {
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

    fn handle_toggle_panel(&mut self, p: &str) {
        /* WHY: Close siblings before opening a new side panel */
        let open = match p {
            "export" => self.state.layout.show_export_panel,
            "story" => self.state.layout.show_story_panel,
            "tools" => self.state.layout.show_tools_panel,
            "toc" => self.state.layout.show_toc,
            _ => false,
        };
        if !open {
            self.state.layout.show_export_panel = false;
            self.state.layout.show_story_panel = false;
            self.state.layout.show_tools_panel = false;
            self.state.layout.show_toc = false;
        }
        match p {
            "export" => self.state.layout.show_export_panel ^= true,
            "story" => self.state.layout.show_story_panel ^= true,
            "tools" => self.state.layout.show_tools_panel ^= true,
            "toc" => self.state.layout.show_toc ^= true,
            _ => {}
        }
    }
}
