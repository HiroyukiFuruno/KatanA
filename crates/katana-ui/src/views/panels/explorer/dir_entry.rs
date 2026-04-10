use super::tree_entry::TreeEntryNode;
use crate::shell::{
    TREE_FONT_SIZE, TREE_HOVER_GAMMA, TREE_HOVER_ROUNDING, TREE_ICON_ARROW_GAP,
    TREE_ICON_LABEL_GAP, TREE_INDENT_STEP, TREE_ROW_HEIGHT,
};
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
            egui::Sense::click(),
        );
        resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

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
            if ui.rect_contains_pointer(rect) && ui.is_enabled() {
                let hover_color = ui
                    .visuals()
                    .widgets
                    .hovered
                    .bg_fill
                    .gamma_multiply(TREE_HOVER_GAMMA);
                ui.painter()
                    .rect_filled(rect, TREE_HOVER_ROUNDING, hover_color);
            }

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child_ui.spacing_mut().item_spacing.x = 0.0;

            let indent = ctx.depth as f32 * TREE_INDENT_STEP;
            child_ui.add_space(indent);

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

        if state.is_open() {
            let prev_depth = ctx.depth;
            ctx.depth += 1;
            for child in children {
                TreeEntryNode::new(child, ctx).show(ui);
            }
            ctx.depth = prev_depth;
        }
    }
}
