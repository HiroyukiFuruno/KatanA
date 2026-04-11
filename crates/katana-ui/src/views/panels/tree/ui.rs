use super::types::*;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

impl<'a, 'b, 'c> TreeContextMenu<'a, 'b, 'c> {
    pub fn new(
        path: &'a std::path::Path,
        is_dir: bool,
        children: Option<&'a [katana_core::workspace::TreeEntry]>,
        entry: Option<&'a katana_core::workspace::TreeEntry>,
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self {
            path,
            is_dir,
            children,
            entry,
            ctx,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let path = self.path;
        let is_dir = self.is_dir;
        let children = self.children;
        let entry = self.entry;
        let ctx = self.ctx;
        let msg = &crate::i18n::I18nOps::get().action;

        let target_dir = if is_dir {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };

        if is_dir {
            if ui.button(msg.new_file.clone()).clicked() {
                *ctx.action = crate::app_state::AppAction::RequestNewFile(target_dir.clone());
                ui.close();
            }
            if ui.button(msg.new_directory.clone()).clicked() {
                *ctx.action = crate::app_state::AppAction::RequestNewDirectory(target_dir);
                ui.close();
            }
            ui.separator();

            if ui
                .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
                .clicked()
            {
                *ctx.action = crate::app_state::AppAction::OpenWorkspace(path.to_path_buf());
                ui.close();
            }
            ui.separator();
        }

        if is_dir {
            let Some(children) = children else {
                return;
            };
            if ui.button(msg.recursive_expand.clone()).clicked() {
                let to_expand: Vec<_> = children
                    .iter()
                    .flat_map(|child| {
                        let mut v = Vec::new();
                        child.collect_all_directory_paths(&mut v);
                        v
                    })
                    .collect();
                ctx.expanded_directories.insert(path.to_path_buf());
                ctx.expanded_directories.extend(to_expand);
                ui.close();
            }
            if ui.button(msg.recursive_open_all.clone()).clicked() {
                let to_open: Vec<_> = children
                    .iter()
                    .flat_map(|child| {
                        let mut v = Vec::new();
                        child.collect_all_markdown_file_paths(&mut v);
                        v
                    })
                    .collect();
                if !to_open.is_empty() {
                    *ctx.action = crate::app_state::AppAction::OpenMultipleDocuments(to_open);
                }
                ui.close();
            }
            crate::widgets::MenuButtonOps::show(
                ui,
                &crate::i18n::I18nOps::get().tab.add_to_group,
                |ui| {
                    super::tab_group_menu::TabGroupMenu::render(
                        ui,
                        path,
                        ctx,
                        true,
                        Some(children),
                    );
                },
            );
        } else if entry.is_some() {
            #[allow(clippy::collapsible_if)]
            if ui.button(msg.open.clone()).clicked() {
                *ctx.action = crate::app_state::AppAction::SelectDocument(path.to_path_buf());
                ui.close();
            }

            crate::widgets::MenuButtonOps::show(
                ui,
                &crate::i18n::I18nOps::get().tab.add_to_group,
                |ui| {
                    super::tab_group_menu::TabGroupMenu::render(ui, path, ctx, false, None);
                },
            );
        }

        ui.separator();

        if ui.button(msg.reveal_in_os.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::RevealInOs(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.copy_path.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::CopyPathToClipboard(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.copy_relative_path.clone()).clicked() {
            *ctx.action =
                crate::app_state::AppAction::CopyRelativePathToClipboard(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.show_meta_info.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::ShowMetaInfo(path.to_path_buf());
            ui.close();
        }

        ui.separator();

        if ui.button(msg.rename.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::RequestRename(path.to_path_buf());
            ui.close();
        }
        if ui.button(msg.delete.clone()).clicked() {
            *ctx.action = crate::app_state::AppAction::RequestDelete(path.to_path_buf());
            ui.close();
        }
    }
}
