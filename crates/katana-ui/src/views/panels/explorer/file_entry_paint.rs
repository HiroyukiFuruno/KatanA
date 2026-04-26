use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, TREE_DRAG_GHOST_GAMMA, TREE_FONT_SIZE, TREE_HOVER_GAMMA,
    TREE_HOVER_ROUNDING, TREE_ICON_ARROW_GAP, TREE_ICON_LABEL_GAP, TREE_INDENT_STEP,
};
use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct FileEntryPaintOps;

impl FileEntryPaintOps {
    pub(crate) fn paint_background(
        ui: &egui::Ui,
        full_rect: egui::Rect,
        is_dragged: bool,
        is_active: bool,
    ) {
        if is_dragged {
            let color = ui
                .visuals()
                .selection
                .bg_fill
                .gamma_multiply(TREE_DRAG_GHOST_GAMMA);
            ui.painter()
                .rect_filled(full_rect, ACTIVE_FILE_HIGHLIGHT_ROUNDING, color);
            return;
        }
        if is_active {
            ui.painter().rect_filled(
                full_rect,
                ACTIVE_FILE_HIGHLIGHT_ROUNDING,
                ui.visuals().selection.bg_fill,
            );
            return;
        }
        if ui.rect_contains_pointer(full_rect) && ui.is_enabled() {
            let color = ui
                .visuals()
                .widgets
                .hovered
                .bg_fill
                .gamma_multiply(TREE_HOVER_GAMMA);
            ui.painter()
                .rect_filled(full_rect, TREE_HOVER_ROUNDING, color);
        }
    }

    pub(crate) fn paint_row_content(
        ui: &mut egui::Ui,
        full_rect: egui::Rect,
        ctx: &TreeRenderContext,
        name: String,
        icon: crate::Icon,
        text_color: egui::Color32,
        _is_active: bool,
    ) {
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(full_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.spacing_mut().item_spacing.x = 0.0;
        child_ui.add_space(ctx.depth as f32 * TREE_INDENT_STEP);
        let arrow_width = crate::icon::IconSize::Small.to_vec2().x;
        child_ui.add_space(arrow_width + TREE_ICON_ARROW_GAP);
        child_ui.visuals_mut().override_text_color = Some(text_color);
        let img = icon.ui_image(&child_ui, crate::icon::IconSize::Medium);
        child_ui.add(img);
        child_ui.add_space(TREE_ICON_LABEL_GAP);
        let font_id = egui::FontId::proportional(TREE_FONT_SIZE);
        let text_pos = child_ui.cursor().left_center();
        child_ui.painter().text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            name,
            font_id,
            text_color,
        );
    }

    pub(crate) fn paint_drop_target(
        ui: &mut egui::Ui,
        full_rect: egui::Rect,
        target_dir: &std::path::Path,
        ctx: &TreeRenderContext,
        response: &egui::Response,
        is_dragged: bool,
    ) {
        let is_pointer_over =
            crate::views::panels::explorer::drag::ExplorerDragUi::is_pointer_over_rect(
                ui, full_rect,
            );
        Self::paint_drop_target_background(ui, full_rect, response, is_dragged);
        if is_pointer_over || is_dragged || response.hovered() {
            crate::views::panels::explorer::drag::ExplorerDragUi::render_drop_target_hint(
                ui,
                full_rect,
                target_dir,
                ctx.ws_root,
                true,
            );
        }
    }

    fn paint_drop_target_background(
        ui: &egui::Ui,
        full_rect: egui::Rect,
        response: &egui::Response,
        is_dragged: bool,
    ) {
        if is_dragged {
            ui.painter().rect_filled(
                full_rect,
                TREE_HOVER_ROUNDING,
                ui.visuals()
                    .selection
                    .bg_fill
                    .gamma_multiply(TREE_DRAG_GHOST_GAMMA),
            );
            return;
        }
        if response.hovered() {
            ui.painter().rect_filled(
                full_rect,
                TREE_HOVER_ROUNDING,
                ui.visuals()
                    .widgets
                    .hovered
                    .bg_fill
                    .gamma_multiply(TREE_HOVER_GAMMA),
            );
        }
    }
}
