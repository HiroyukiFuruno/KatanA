use crate::app_state::AppAction;
use crate::shell::TREE_ROW_HEIGHT;
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct ExplorerRootDropArea;

impl ExplorerRootDropArea {
    pub(crate) fn show(ui: &mut egui::Ui, ctx: &mut TreeRenderContext, ws_root: &std::path::Path) {
        let drag_target_dir =
            egui::DragAndDrop::payload::<std::path::PathBuf>(ui.ctx()).and_then(|source_path| {
                crate::views::panels::explorer::drag::ExplorerDragUi::resolve_drop_target_dir_for_empty_tree_area(
                    source_path.as_path(),
                    ws_root,
                )
            });
        let (rect, resp) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), TREE_ROW_HEIGHT * 2.0),
            egui::Sense::click_and_drag(),
        );
        resp.context_menu(|ui| {
            Self::show_context_menu(ui, ctx, ws_root);
        });
        if let Some(source_path) = resp.dnd_release_payload::<std::path::PathBuf>()
            && let Some(target_dir) = drag_target_dir.as_ref()
        {
            *ctx.action = AppAction::RequestMoveFsNode {
                source_path: (*source_path).clone(),
                target_dir: target_dir.clone(),
            };
        }
        Self::show_drop_hint(ui, rect, drag_target_dir.as_deref(), ws_root);
    }

    fn show_context_menu(
        ui: &mut egui::Ui,
        ctx: &mut TreeRenderContext,
        ws_root: &std::path::Path,
    ) {
        let msg = &crate::i18n::I18nOps::get().action;
        if ui.button(msg.format_workspace_markdown.clone()).clicked() {
            *ctx.action = AppAction::FormatWorkspaceMarkdown(ws_root.to_path_buf());
            ui.close();
        }
        if ui.button(msg.new_file.clone()).clicked() {
            *ctx.action = AppAction::RequestNewFile(ws_root.to_path_buf());
            ui.close();
        }
        if ui.button(msg.new_directory.clone()).clicked() {
            *ctx.action = AppAction::RequestNewDirectory(ws_root.to_path_buf());
            ui.close();
        }
    }

    fn show_drop_hint(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        target_dir: Option<&std::path::Path>,
        ws_root: &std::path::Path,
    ) {
        let Some(target_dir) = target_dir else {
            return;
        };
        let is_pointer_over =
            crate::views::panels::explorer::drag::ExplorerDragUi::is_pointer_over_rect(ui, rect);
        if !is_pointer_over {
            return;
        }
        crate::views::panels::explorer::drag::ExplorerDragUi::render_drop_target_hint(
            ui,
            rect,
            target_dir,
            Some(ws_root),
            true,
        );
        ui.painter().rect_filled(
            rect,
            crate::shell::TREE_HOVER_ROUNDING,
            ui.visuals()
                .widgets
                .hovered
                .bg_fill
                .gamma_multiply(crate::shell::TREE_HOVER_GAMMA),
        );
    }
}
