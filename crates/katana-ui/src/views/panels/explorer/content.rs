use super::file_entry::FileEntryNode;
use super::tree_entry::TreeEntryNode;
use super::types::ExplorerLogicOps;
use crate::app_state::AppAction;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct ExplorerContent<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub histories: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
    pub action: &'a mut AppAction,
}

impl<'a> ExplorerContent<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        histories: &'a [String],
        active_path: Option<&'a std::path::Path>,
        tab_groups: &'a [crate::state::document::TabGroup],
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            histories,
            active_path,
            tab_groups,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        if self.workspace.data.is_some() {
            self.show_active_workspace(ui);
        } else {
            crate::views::panels::explorer::empty::EmptyWorkspaceView::new(
                self.histories,
                self.action,
            )
            .show(ui);
        }
    }

    fn show_active_workspace(self, ui: &mut egui::Ui) {
        let (workspace, search, active_path, action) =
            (self.workspace, self.search, self.active_path, self.action);
        let ws = workspace.data.as_ref().unwrap();
        let entries = ws.tree.clone();
        let ws_root = ws.root.clone();

        ExplorerLogicOps::update_tree_expansion(workspace);
        ExplorerLogicOps::update_search_filter_cache(search, &ws_root, &entries);

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
                    tab_groups: Some(self.tab_groups),
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
                if let Some(fs) = ctx.filter_set
                    && !fs.contains(path)
                {
                    continue;
                }
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
}
