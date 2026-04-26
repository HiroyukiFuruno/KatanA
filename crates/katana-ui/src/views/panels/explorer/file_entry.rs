use crate::shell::TREE_ROW_HEIGHT;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct FileEntryNode<'a, 'b, 'c> {
    pub entry: &'a katana_core::workspace::TreeEntry,
    pub path: &'a std::path::Path,
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

impl<'a, 'b, 'c> FileEntryNode<'a, 'b, 'c> {
    pub fn new(
        entry: &'a katana_core::workspace::TreeEntry,
        path: &'a std::path::Path,
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self { entry, path, ctx }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let entry = self.entry;
        let path = self.path;
        let ctx = self.ctx;
        let name = if ctx.is_flat_view {
            crate::shell_logic::ShellLogicOps::relative_full_path(path, ctx.ws_root)
        } else {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string()
        };

        let accessible_label = format!("file {}", name);

        let is_active = ctx.active_path.is_some_and(|ap| ap == path);

        let text_color = if is_active {
            ui.visuals().widgets.active.fg_stroke.color
        } else {
            ui.visuals().text_color()
        };
        let (full_rect, mut resp) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), TREE_ROW_HEIGHT),
            egui::Sense::click_and_drag(),
        );
        let is_dragged = ui.ctx().is_being_dragged(resp.id);
        resp = if is_dragged {
            resp.on_hover_cursor(egui::CursorIcon::Grabbing)
        } else {
            resp.on_hover_cursor(egui::CursorIcon::Grab)
        };
        let icon = if entry.is_markdown() {
            crate::icon::Icon::Markdown
        } else if entry.is_image() {
            crate::icon::Icon::Image
        } else {
            crate::icon::Icon::Document
        };
        let drag_target_dir =
            egui::DragAndDrop::payload::<std::path::PathBuf>(ui.ctx()).and_then(|source_path| {
                crate::views::panels::explorer::drag::ExplorerDragUi::resolve_drop_target_dir(
                    source_path.as_path(),
                    path,
                    false,
                )
            });
        if resp.drag_started() {
            resp.dnd_set_drag_payload(path.to_path_buf());
        }
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &accessible_label)
        });
        if resp.drag_started() || is_dragged {
            crate::views::panels::explorer::drag::ExplorerDragUi::render_drag_ghost(
                ui,
                path,
                false,
                full_rect,
                ctx.ws_root,
                icon,
            );
        }

        if ui.is_rect_visible(full_rect) {
            crate::views::panels::explorer::file_entry_paint::FileEntryPaintOps::paint_background(
                ui, full_rect, is_dragged, is_active,
            );
            crate::views::panels::explorer::file_entry_paint::FileEntryPaintOps::paint_row_content(
                ui, full_rect, ctx, name, icon, text_color, is_active,
            );
        }

        if !ctx.disable_context_menu {
            resp.context_menu(|ui| {
                crate::views::panels::tree::TreeContextMenu::new(
                    path,
                    false,
                    None,
                    Some(entry),
                    ctx,
                )
                .show(ui);
            });
        }

        if let Some(source_path) = resp.dnd_release_payload::<std::path::PathBuf>()
            && let Some(target_dir) =
                crate::views::panels::explorer::drag::ExplorerDragUi::resolve_drop_target_dir(
                    source_path.as_path(),
                    path,
                    false,
                )
        {
            *ctx.action = crate::app_state::AppAction::RequestMoveFsNode {
                source_path: (*source_path).clone(),
                target_dir,
            };
        }

        if resp.clicked() {
            *ctx.action = crate::app_state::AppAction::SelectDocument(path.to_path_buf());
        }

        if !ui.is_rect_visible(full_rect) {
            return;
        }

        if let Some(target_dir) = drag_target_dir {
            crate::views::panels::explorer::file_entry_paint::FileEntryPaintOps::paint_drop_target(
                ui,
                full_rect,
                &target_dir,
                ctx,
                &resp,
                is_dragged,
            );
        }
    }
}
