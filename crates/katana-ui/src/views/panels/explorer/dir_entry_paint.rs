use crate::shell::{
    TREE_ACCORDION_LINE_GAMMA, TREE_ACCORDION_LINE_OFFSET, TREE_ACCORDION_LINE_WIDTH,
    TREE_DRAG_GHOST_GAMMA, TREE_FONT_SIZE, TREE_HOVER_GAMMA, TREE_HOVER_ROUNDING,
    TREE_ICON_ARROW_GAP, TREE_ICON_LABEL_GAP, TREE_INDENT_STEP,
};
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct DirectoryEntryPaintOps;

impl DirectoryEntryPaintOps {
    pub(crate) fn paint_background_and_drop_hint(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        drag_target_dir: Option<&std::path::Path>,
        ctx: &TreeRenderContext,
    ) {
        let is_dragging_this_frame = ui.ctx().is_being_dragged(response.id);
        if is_dragging_this_frame {
            Self::paint_dragging_background(ui, rect);
            Self::paint_drop_hint_if_needed(ui, rect, response, drag_target_dir, ctx, true);
            return;
        }
        if let Some(target_dir) = drag_target_dir {
            if response.hovered() {
                Self::paint_hover_background(ui, rect);
                Self::paint_drop_hint(ui, rect, target_dir, ctx);
                return;
            }
            if crate::views::panels::explorer::drag::ExplorerDragUi::is_pointer_over_rect(ui, rect)
            {
                Self::paint_drop_hint(ui, rect, target_dir, ctx);
                return;
            }
        }
        if ui.rect_contains_pointer(rect) && ui.is_enabled() {
            Self::paint_hover_background(ui, rect);
        }
    }

    pub(crate) fn paint_row(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        ctx: &TreeRenderContext,
        name: &str,
        is_open: bool,
        file_tree_color: egui::Color32,
    ) {
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.spacing_mut().item_spacing.x = 0.0;
        child_ui.add_space(ctx.depth as f32 * TREE_INDENT_STEP);
        let arrow_icon = if is_open {
            crate::icon::Icon::ChevronDown
        } else {
            crate::icon::Icon::ChevronRight
        };
        let folder_icon = if is_open {
            crate::icon::Icon::FolderOpen
        } else {
            crate::icon::Icon::FolderClosed
        };
        child_ui.visuals_mut().override_text_color = Some(file_tree_color);
        let img_arrow = arrow_icon.ui_image(&child_ui, crate::icon::IconSize::Small);
        child_ui.add(img_arrow);
        child_ui.add_space(TREE_ICON_ARROW_GAP);
        let img_folder = folder_icon.ui_image(&child_ui, crate::icon::IconSize::Medium);
        child_ui.add(img_folder);
        child_ui.add_space(TREE_ICON_LABEL_GAP);
        child_ui.add(
            egui::Label::new(
                egui::RichText::new(name)
                    .color(file_tree_color)
                    .size(TREE_FONT_SIZE),
            )
            .selectable(false)
            .truncate(),
        );
    }

    pub(crate) fn paint_vertical_line(
        ui: &egui::Ui,
        rect: egui::Rect,
        line_start_y: f32,
        line_end_y: f32,
        depth: usize,
    ) {
        let indent_x = rect.left() + (depth as f32) * TREE_INDENT_STEP;
        let line_x = indent_x + TREE_ACCORDION_LINE_OFFSET;
        let stroke = egui::Stroke::new(
            TREE_ACCORDION_LINE_WIDTH,
            ui.visuals()
                .text_color()
                .gamma_multiply(TREE_ACCORDION_LINE_GAMMA),
        );
        ui.painter().line_segment(
            [
                egui::pos2(line_x, line_start_y),
                egui::pos2(line_x, line_end_y),
            ],
            stroke,
        );
    }

    fn paint_dragging_background(ui: &egui::Ui, rect: egui::Rect) {
        ui.painter().rect_filled(
            rect,
            TREE_HOVER_ROUNDING,
            ui.visuals()
                .selection
                .bg_fill
                .gamma_multiply(TREE_DRAG_GHOST_GAMMA),
        );
    }

    fn paint_hover_background(ui: &egui::Ui, rect: egui::Rect) {
        ui.painter().rect_filled(
            rect,
            TREE_HOVER_ROUNDING,
            ui.visuals()
                .widgets
                .hovered
                .bg_fill
                .gamma_multiply(TREE_HOVER_GAMMA),
        );
    }

    fn paint_drop_hint_if_needed(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        drag_target_dir: Option<&std::path::Path>,
        ctx: &TreeRenderContext,
        is_dragging_this_frame: bool,
    ) {
        let Some(target_dir) = drag_target_dir else {
            return;
        };
        let is_over =
            crate::views::panels::explorer::drag::ExplorerDragUi::is_pointer_over_rect(ui, rect);
        if response.hovered() || is_over || is_dragging_this_frame {
            Self::paint_drop_hint(ui, rect, target_dir, ctx);
        }
    }

    fn paint_drop_hint(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        target_dir: &std::path::Path,
        ctx: &TreeRenderContext,
    ) {
        crate::views::panels::explorer::drag::ExplorerDragUi::render_drop_target_hint(
            ui,
            rect,
            target_dir,
            ctx.ws_root,
            true,
        );
    }
}
