use crate::app::*;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn dispatch_secondary(&mut self, ctx: &egui::Context, action: AppAction) {
        match action {
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
            AppAction::CheckForUpdates => self.start_update_check(true),
            AppAction::ExportDocument(fmt) => self.handle_export_document(ctx, fmt),
            AppAction::AcceptTerms(version) => {
                self.state
                    .config
                    .settings
                    .settings_mut()
                    .terms_accepted_version = Some(version);
                if !self.state.config.try_save_settings() {
                    tracing::warn!("Failed to save terms acceptance");
                }
            }
            AppAction::DeclineTerms => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            AppAction::ShowMetaInfo(path) => self.show_meta_info_for = Some(path),
            AppAction::SkipVersion(version) => {
                self.state
                    .config
                    .settings
                    .settings_mut()
                    .updates
                    .skipped_version = Some(version);
                let _ = self.state.config.try_save_settings();
                self.show_update_dialog = false;
            }
            AppAction::DismissUpdate => self.show_update_dialog = false,
            AppAction::ConfirmRelaunch => {
                if let Some(_prep) = self.pending_relaunch.take() {
                    #[cfg(all(not(test), not(coverage)))]
                    {
                        let _ = katana_core::update::UpdateInstallerOps::execute_relauncher(_prep);
                        std::process::exit(0);
                    }
                }
            }
            AppAction::ShowReleaseNotes => self.handle_show_release_notes(),
            AppAction::ClearAllCaches => self.handle_action_clear_all_caches(ctx),
            AppAction::RequestNewFile(path) => {
                let ext = self
                    .state
                    .config
                    .settings
                    .settings()
                    .workspace
                    .visible_extensions
                    .first()
                    .cloned();
                self.state.layout.create_fs_node_modal = Some((path, String::new(), ext, false));
            }
            AppAction::RequestNewDirectory(path) => {
                self.state.layout.create_fs_node_modal = Some((path, String::new(), None, true));
            }
            AppAction::RequestRename(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.state.layout.rename_modal = Some((path, name));
            }
            AppAction::RequestDelete(path) => self.state.layout.delete_modal = Some(path),
            AppAction::CopyPathToClipboard(path) => {
                ctx.copy_text(path.to_string_lossy().to_string());
            }
            AppAction::CopyRelativePathToClipboard(path) => {
                let rel_path = if let Some(ws) = &self.state.workspace.data {
                    path.strip_prefix(&ws.root).unwrap_or(&path).to_path_buf()
                } else {
                    path.clone()
                };
                ctx.copy_text(rel_path.to_string_lossy().to_string());
            }
            AppAction::RevealInOs(path) => self.handle_action_reveal_in_os(path),
            AppAction::CreateTabGroup {
                name,
                color_hex,
                initial_member,
            } => self.handle_action_create_tab_group(name, color_hex, vec![initial_member]),
            AppAction::CreateTabGroupMany {
                name,
                color_hex,
                members,
            } => self.handle_action_create_tab_group(name, color_hex, members),
            AppAction::AddTabToGroup { group_id, member } => {
                self.handle_action_add_tabs_to_group(group_id, vec![member])
            }
            AppAction::AddTabsToGroup { group_id, members } => {
                self.handle_action_add_tabs_to_group(group_id, members)
            }
            AppAction::RemoveTabFromGroup(member) => {
                self.handle_action_remove_tab_from_group(member)
            }
            AppAction::RenameTabGroup { group_id, new_name } => {
                self.handle_action_rename_tab_group(group_id, new_name)
            }
            AppAction::ClearInlineRename => self.state.layout.inline_rename_group = None,
            AppAction::RecolorTabGroup {
                group_id,
                new_color,
            } => self.handle_action_recolor_tab_group(group_id, new_color),
            AppAction::CloseTabGroup(group_id) => self.handle_action_close_tab_group(group_id),
            AppAction::UngroupTabGroup(group_id) => {
                self.state.document.tab_groups.retain(|g| g.id != group_id);
                self.save_workspace_state();
            }
            AppAction::ToggleCollapseTabGroup(group_id) => {
                self.handle_action_toggle_collapse_tab_group(group_id)
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
            AppAction::RevealImageAsset(path) => self.handle_action_reveal_image_asset(path),
            AppAction::None => {}
            AppAction::InstallUpdate => self.handle_action_install_update(),
            _ => {}
        }
    }
}
