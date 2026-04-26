use crate::app::download::DownloadOps;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn dispatch_secondary(&mut self, ctx: &egui::Context, action: AppAction) {
        match action {
            AppAction::PickOpenFileInCurrentWorkspace | AppAction::PickOpenFileInNewWorkspace => {
                self.handle_action_pick_open_file(action);
            }
            AppAction::OpenFileInCurrentWorkspace(p) => {
                crate::app::action::FileOpenOps::open_in_current_workspace(self, p);
            }
            AppAction::OpenFileInNewWorkspace(p) => {
                crate::app::action::FileOpenOps::open_as_temporary_workspace(self, p);
            }
            AppAction::OpenDroppedFiles(paths) => {
                crate::app::action::FileOpenOps::open_dropped_files(self, paths);
            }
            AppAction::RequestMoveFsNode {
                source_path,
                target_dir,
            } => self.handle_action_request_move_fs_node(source_path, target_dir),
            AppAction::MoveFsNode {
                source_path,
                target_path,
            } => self.handle_action_move_fs_node(source_path, target_path),
            AppAction::CloseOtherDocuments(idx) => self.handle_action_close_other_documents(idx),
            AppAction::CloseAllDocuments => self.handle_action_close_all_documents(),
            AppAction::CloseDocumentsToRight(idx) => self.handle_action_close_to_right(idx),
            AppAction::CloseDocumentsToLeft(idx) => self.handle_action_close_to_left(idx),
            AppAction::TogglePinDocument(idx) => self.handle_action_toggle_pin(idx),
            AppAction::RestoreClosedDocument => self.handle_action_restore_closed_document(),
            AppAction::ReorderDocument {
                from,
                to,
                new_group_id,
            } => self.handle_action_reorder_document(from, to, new_group_id),
            AppAction::ReorderActivityRail { from, to } => {
                self.handle_action_reorder_activity_rail(from, to)
            }
            AppAction::StartPlantumlDownload { url, dest } => {
                /* WHY: Reuse the existing DownloadOps pipeline (DownloadRequest → start_download).
                 * This forces a fresh download even when the JAR already exists, enabling
                 * users to update to the latest PlantUML release from the Settings screen. */
                self.start_download(crate::preview_pane::DownloadRequest {
                    tool_name: "PlantUML".to_string(),
                    url,
                    dest,
                });
            }
            AppAction::StartDrawioDownload { url, dest } => {
                self.start_download(crate::preview_pane::DownloadRequest {
                    tool_name: "Draw.io".to_string(),
                    url,
                    dest,
                });
            }
            AppAction::StartMermaidDownload { url, dest } => {
                self.start_download(crate::preview_pane::DownloadRequest {
                    tool_name: "Mermaid".to_string(),
                    url,
                    dest,
                });
            }
            AppAction::OpenHelpDemo => self.handle_action_open_help_demo(),
            AppAction::OpenWelcomeScreen => self.handle_action_open_welcome_screen(),
            AppAction::OpenUserGuide => self.handle_action_open_user_guide(),
            AppAction::OpenGitHub => {
                let _ = open::that(crate::about_info::APP_REPOSITORY);
            }
            AppAction::OpenOfficialWebsite => {
                let _ = open::that(crate::about_info::APP_WEBSITE_URL);
            }
            AppAction::SwitchDemoLanguage(lang) => self.handle_action_switch_demo_lang(&lang),
            /* WHY: Markdown authoring — transform buffer around cursor / selection. */
            AppAction::AuthorMarkdown(op) => self.handle_action_author_markdown(op),
            /* WHY: Image ingest — implemented in Task 2 (stubs for now). */
            AppAction::IngestImageFile => self.handle_action_ingest_image_file(),
            AppAction::IngestClipboardImage => self.handle_action_ingest_clipboard_image(),
            AppAction::SetSplitDirection(dir) => self.state.set_active_split_direction(dir),
            AppAction::SetPaneOrder(order) => self.state.set_active_pane_order(order),
            AppAction::SetViewMode(mode) => self.state.set_active_view_mode(mode),
            AppAction::ToggleScrollSync(is_on) => {
                self.state.scroll.sync_override = Some(is_on);
            }
            AppAction::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            AppAction::None => {}
            AppAction::InstallUpdate => self.handle_action_install_update(),
            AppAction::OpenDocSearch => {
                self.state.search.doc_search_open = true;
                ctx.memory_mut(|m| {
                    m.data
                        .insert_temp(egui::Id::new("search_newly_opened"), true)
                });
            }
            AppAction::ToggleDocSearch => {
                if !self.state.search.doc_search_open {
                    self.state.search.doc_search_open = true;
                    ctx.memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new("search_newly_opened"), true)
                    });
                    self.trigger_action(AppAction::DocSearchQueryChanged);
                } else {
                    self.state.search.doc_search_open = false;
                    self.state.search.doc_search_matches.clear();
                }
            }
            AppAction::DocSearchQueryChanged => self.handle_action_doc_search_changed(),
            AppAction::DocSearchNext => self.handle_action_doc_search_next(ctx),
            AppAction::DocSearchPrev => self.handle_action_doc_search_prev(ctx),
            AppAction::ToggleProblemsPanel => self.state.diagnostics.is_panel_open ^= true,
            AppAction::RefreshDiagnostics => self.handle_action_refresh_diagnostics(),
            AppAction::FormatMarkdownFile(path) => {
                self.handle_action_format_markdown_file(ctx, path)
            }
            AppAction::FormatWorkspaceMarkdown(root) => {
                self.handle_action_format_workspace_markdown(ctx, root)
            }
            AppAction::ToggleExplorerFilter => {
                let current = self.state.search.filter_enabled;
                self.state.search.filter_enabled = !current;
                if !current {
                    ctx.memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new("filter_newly_enabled"), true)
                    });
                }
            }
            AppAction::ToggleSplitMode => {
                self.state.set_active_view_mode(ViewMode::Split);
            }
            AppAction::ToggleCodePreview => {
                let next_mode = match self.state.active_view_mode() {
                    ViewMode::Split => ViewMode::PreviewOnly,
                    ViewMode::PreviewOnly => ViewMode::CodeOnly,
                    ViewMode::CodeOnly => ViewMode::PreviewOnly,
                };
                self.state.set_active_view_mode(next_mode);
            }
            AppAction::SelectNextTab => self.handle_action_next_tab(),
            AppAction::SelectPrevTab => self.handle_action_prev_tab(),
            AppAction::ZoomIn => {
                const ZOOM_STEP: f32 = 0.1;
                let current = ctx.zoom_factor();
                ctx.set_zoom_factor(current + ZOOM_STEP);
            }
            AppAction::ZoomOut => {
                const ZOOM_STEP: f32 = 0.1;
                const ZOOM_MIN: f32 = 0.2;
                let current = ctx.zoom_factor();
                if current > ZOOM_MIN {
                    ctx.set_zoom_factor(current - ZOOM_STEP);
                }
            }
            _ => self.dispatch_tertiary(ctx, action),
        }
    }
}
