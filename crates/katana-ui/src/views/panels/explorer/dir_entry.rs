use super::tree_entry::TreeEntryNode;
use crate::shell::TREE_ROW_HEIGHT;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct DirectoryEntryNode<'a, 'b, 'c> {
    pub path: &'a std::path::Path,
    pub children: &'a [katana_core::workspace::TreeEntry],
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

impl<'a, 'b, 'c> DirectoryEntryNode<'a, 'b, 'c> {
    pub fn new(
        path: &'a std::path::Path,
        children: &'a [katana_core::workspace::TreeEntry],
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self {
            path,
            children,
            ctx,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let path = self.path;
        let children = self.children;
        let ctx = self.ctx;
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let id = ui.make_persistent_id(format!("dir:{}", path.display()));

        let is_open = ctx.expanded_directories.contains(path);

        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, is_open);
        state.set_open(is_open);
        let file_tree_color = ui.visuals().text_color();
        let (rect, mut resp) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), TREE_ROW_HEIGHT),
            egui::Sense::click_and_drag(),
        );
        resp = if ui.ctx().is_being_dragged(resp.id) {
            resp.on_hover_cursor(egui::CursorIcon::Grabbing)
        } else {
            resp.on_hover_cursor(egui::CursorIcon::Grab)
        };
        if resp.drag_started() {
            resp.dnd_set_drag_payload(path.to_path_buf());
        }
        let drag_target_dir =
            egui::DragAndDrop::payload::<std::path::PathBuf>(ui.ctx()).and_then(|source_path| {
                crate::views::panels::explorer::drag::ExplorerDragUi::resolve_drop_target_dir(
                    source_path.as_path(),
                    path,
                    true,
                )
            });
        if resp.drag_started() || ui.ctx().is_being_dragged(resp.id) {
            crate::views::panels::explorer::drag::ExplorerDragUi::render_drag_ghost(
                ui,
                path,
                true,
                rect,
                ctx.ws_root,
                if is_open {
                    crate::icon::Icon::FolderOpen
                } else {
                    crate::icon::Icon::FolderClosed
                },
            );
        }
        if let Some(source_path) = resp.dnd_release_payload::<std::path::PathBuf>()
            && let Some(target_dir) =
                crate::views::panels::explorer::drag::ExplorerDragUi::resolve_drop_target_dir(
                    source_path.as_path(),
                    path,
                    true,
                )
        {
            *ctx.action = crate::app_state::AppAction::RequestMoveFsNode {
                source_path: (*source_path).clone(),
                target_dir,
            };
        }

        let accessible_label = format!("dir {}", name);
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &accessible_label)
        });

        if resp.clicked() {
            if is_open {
                ctx.expanded_directories.remove(path);
            } else {
                ctx.expanded_directories.insert(path.to_path_buf());
            }
        }

        if ui.is_rect_visible(rect) {
            crate::views::panels::explorer::dir_entry_paint::DirectoryEntryPaintOps::paint_background_and_drop_hint(
                ui,
                rect,
                &resp,
                drag_target_dir.as_deref(),
                ctx,
            );
            crate::views::panels::explorer::dir_entry_paint::DirectoryEntryPaintOps::paint_row(
                ui,
                rect,
                ctx,
                name,
                is_open,
                file_tree_color,
            );
        }

        if !ctx.disable_context_menu {
            resp.context_menu(|ui| {
                crate::views::panels::tree::TreeContextMenu::new(
                    path,
                    true,
                    Some(children),
                    None,
                    ctx,
                )
                .show(ui);
            });
        }

        if resp.clicked() {
            let new_state = !is_open;
            state.set_open(new_state);
            if new_state {
                ctx.expanded_directories.insert(path.to_path_buf());
            } else {
                ctx.expanded_directories.remove(path);
            }
        }
        state.store(ui.ctx());

        let line_start_y = rect.bottom();
        if state.is_open() {
            let prev_depth = ctx.depth;
            ctx.depth += 1;
            for child in children {
                TreeEntryNode::new(child, ctx).show(ui);
            }
            let line_end_y = ui.cursor().top();
            ctx.depth = prev_depth;

            if ctx.show_vertical_line {
                crate::views::panels::explorer::dir_entry_paint::DirectoryEntryPaintOps::paint_vertical_line(
                    ui,
                    rect,
                    line_start_y,
                    line_end_y,
                    ctx.depth,
                );
            }
        }
    }
}
