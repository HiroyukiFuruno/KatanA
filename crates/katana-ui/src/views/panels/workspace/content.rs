use super::file_entry::FileEntryNode;
use super::tree_entry::TreeEntryNode;
use super::types::WorkspaceLogicOps;
use crate::app_state::AppAction;
use crate::shell::{
    NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_CLOSE_BUTTON_WIDTH,
    RECENT_WORKSPACES_ITEM_SPACING, RECENT_WORKSPACES_SPACING,
};
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct WorkspaceContent<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceContent<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        recent_paths: &'a [String],
        active_path: Option<&'a std::path::Path>,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            recent_paths,
            active_path,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        if self.workspace.data.is_some() {
            self.show_active_workspace(ui);
        } else {
            self.show_empty_workspace(ui);
        }
    }

    fn show_active_workspace(self, ui: &mut egui::Ui) {
        let (workspace, search, active_path, action) =
            (self.workspace, self.search, self.active_path, self.action);
        let ws = workspace.data.as_ref().unwrap();
        let entries = ws.tree.clone();
        let ws_root = ws.root.clone();

        WorkspaceLogicOps::update_tree_expansion(workspace);
        WorkspaceLogicOps::update_search_filter_cache(search, &ws_root, &entries);

        let filter_set = search.filter_cache.as_ref().map(|(_, v)| v);

        egui::ScrollArea::vertical()
            .id_salt("workspace_tree_scroll")
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                let is_flat_view = workspace.is_flat_view(&ws_root);
                let mut ctx = TreeRenderContext {
                    action,
                    depth: 0,
                    active_path,
                    filter_set,
                    expanded_directories: &mut workspace.expanded_directories,
                    disable_context_menu: false,
                    is_flat_view,
                    ws_root: Some(&ws_root),
                };

                if is_flat_view {
                    Self::show_flat_view(ui, &entries, &mut ctx);
                } else {
                    for entry in &entries {
                        TreeEntryNode::new(entry, &mut ctx).show(ui);
                    }
                }
            });
    }

    fn show_flat_view(
        ui: &mut egui::Ui,
        entries: &[katana_core::workspace::TreeEntry],
        ctx: &mut TreeRenderContext,
    ) {
        let mut flat_entries = Vec::new();
        Self::collect_files(entries, &mut flat_entries);
        for entry in flat_entries {
            if let katana_core::workspace::TreeEntry::File { path } = entry {
                FileEntryNode::new(entry, path, ctx).show(ui);
            }
        }
    }

    fn collect_files<'b>(
        entries: &'b [katana_core::workspace::TreeEntry],
        out: &mut Vec<&'b katana_core::workspace::TreeEntry>,
    ) {
        for e in entries {
            match e {
                katana_core::workspace::TreeEntry::File { .. } => out.push(e),
                katana_core::workspace::TreeEntry::Directory { children, .. } => {
                    Self::collect_files(children, out)
                }
            }
        }
    }

    fn show_empty_workspace(self, ui: &mut egui::Ui) {
        ui.label(
            crate::i18n::I18nOps::get()
                .workspace
                .no_workspace_open
                .clone(),
        );
        ui.add_space(NO_WORKSPACE_BOTTOM_SPACING);

        if ui
            .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
            .clicked()
        {
            if let Some(path) = crate::shell_ui::ShellUiOps::open_folder_dialog() {
                *self.action = AppAction::OpenWorkspace(path);
            }
        }

        if !self.recent_paths.is_empty() {
            ui.add_space(RECENT_WORKSPACES_SPACING);
            ui.separator();
            ui.add_space(RECENT_WORKSPACES_SPACING);
            ui.heading(
                crate::i18n::I18nOps::get()
                    .workspace
                    .recent_workspaces
                    .clone(),
            );
            ui.add_space(RECENT_WORKSPACES_ITEM_SPACING);

            Self::show_recent_paths(ui, self.recent_paths, self.action);
        }
    }

    fn show_recent_paths(ui: &mut egui::Ui, recent_paths: &[String], action: &mut AppAction) {
        for path in recent_paths.iter().rev() {
            let mut open_clicked = false;
            let mut remove_clicked = false;
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let btn_width = RECENT_WORKSPACES_CLOSE_BUTTON_WIDTH;
                let item_spacing = ui.spacing().item_spacing.x;
                let label_width = (ui.available_width() - btn_width - item_spacing).max(0.0);

                let open_resp = ui.add_sized(
                    egui::vec2(label_width, ui.spacing().interact_size.y),
                    egui::Button::selectable(false, path.as_str()).frame_when_inactive(true),
                );
                open_clicked = open_resp.clicked();

                let remove_resp = ui
                    .add(
                        egui::Button::new("×")
                            .frame(false)
                            .min_size(egui::vec2(btn_width, ui.spacing().interact_size.y)),
                    )
                    .on_hover_text(crate::i18n::I18nOps::get().action.remove_workspace.clone());
                remove_clicked = remove_resp.clicked();
            });

            if open_clicked {
                *action = AppAction::OpenWorkspace(std::path::PathBuf::from(path));
            } else if remove_clicked {
                *action = AppAction::RemoveWorkspace(path.clone());
            }
        }
    }
}
