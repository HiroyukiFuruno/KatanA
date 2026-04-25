use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, TREE_FONT_SIZE, TREE_HOVER_GAMMA, TREE_HOVER_ROUNDING,
    TREE_ICON_ARROW_GAP, TREE_ICON_LABEL_GAP, TREE_INDENT_STEP, TREE_ROW_HEIGHT,
};
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
            egui::Sense::click(),
        );
        resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

        if ui.is_rect_visible(full_rect) {
            if is_active {
                let highlight_color = ui.visuals().selection.bg_fill;
                ui.painter().rect_filled(
                    full_rect,
                    ACTIVE_FILE_HIGHLIGHT_ROUNDING,
                    highlight_color,
                );
            } else if ui.rect_contains_pointer(full_rect) && ui.is_enabled() {
                let hover_color = ui
                    .visuals()
                    .widgets
                    .hovered
                    .bg_fill
                    .gamma_multiply(TREE_HOVER_GAMMA);
                ui.painter()
                    .rect_filled(full_rect, TREE_HOVER_ROUNDING, hover_color);
            }

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(full_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child_ui.spacing_mut().item_spacing.x = 0.0;

            let indent = ctx.depth as f32 * TREE_INDENT_STEP;
            child_ui.add_space(indent);

            let arrow_width = crate::icon::IconSize::Small.to_vec2().x;
            child_ui.add_space(arrow_width + TREE_ICON_ARROW_GAP);

            child_ui.visuals_mut().override_text_color = Some(text_color);

            let icon = if entry.is_markdown() {
                crate::icon::Icon::Markdown
            } else if entry.is_image() {
                crate::icon::Icon::Image
            } else {
                crate::icon::Icon::Document
            };

            let img = icon.ui_image(&child_ui, crate::icon::IconSize::Medium);
            child_ui.add(img);
            child_ui.add_space(TREE_ICON_LABEL_GAP);

            let mut rich = egui::RichText::new(name)
                .color(text_color)
                .size(TREE_FONT_SIZE);
            if is_active {
                rich = rich.strong();
            }
            let resp_label = child_ui.add(egui::Label::new(rich).truncate().selectable(false));

            resp_label.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &accessible_label)
            });
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

        if resp.clicked() {
            *ctx.action = crate::app_state::AppAction::SelectDocument(path.to_path_buf());
        }
    }
}
