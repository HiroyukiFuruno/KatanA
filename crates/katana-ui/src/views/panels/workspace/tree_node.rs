use crate::shell::{ACTIVE_FILE_HIGHLIGHT_ROUNDING, TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT};
use crate::shell_ui::{TreeRenderContext, indent_prefix};
use eframe::egui;

pub(crate) struct TreeEntryNode<'a, 'b, 'c> {
    pub entry: &'a katana_core::workspace::TreeEntry,
    pub ctx: &'a mut TreeRenderContext<'b, 'c>,
}

impl<'a, 'b, 'c> TreeEntryNode<'a, 'b, 'c> {
    pub fn new(
        entry: &'a katana_core::workspace::TreeEntry,
        ctx: &'a mut TreeRenderContext<'b, 'c>,
    ) -> Self {
        Self { entry, ctx }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let entry = self.entry;
        let ctx = self.ctx;
        use katana_core::workspace::TreeEntry;
        let entry_path = match entry {
            TreeEntry::Directory { path, .. } => path,
            TreeEntry::File { path } => path,
        };
        if let Some(fs) = ctx.filter_set {
            if !fs.contains(entry_path) {
                return;
            }
        }
        match entry {
            TreeEntry::Directory { path, children } => {
                DirectoryEntryNode::new(path, children, ctx).show(ui);
            }
            TreeEntry::File { path } => {
                FileEntryNode::new(entry, path, ctx).show(ui);
            }
        }
    }
}

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
                ui.painter()
                    .rect_filled(rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
            }

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child_ui.add_space(TREE_LABEL_HOFFSET);
            let prefix = indent_prefix(ctx.depth);
            let arrow_icon = if is_open {
                crate::icon::Icon::PanDown
            } else {
                crate::icon::Icon::PanRight
            };
            let folder_icon = if is_open {
                crate::icon::Icon::FolderOpen
            } else {
                crate::icon::Icon::FolderClosed
            };

            child_ui.add(egui::Label::new(prefix).selectable(false));

            child_ui.add(
                arrow_icon
                    .image(crate::icon::IconSize::Small)
                    .tint(file_tree_color),
            );
            child_ui.add(
                folder_icon
                    .image(crate::icon::IconSize::Medium)
                    .tint(file_tree_color),
            );
            child_ui.add(
                egui::Label::new(egui::RichText::new(name).color(file_tree_color))
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
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

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
                ui.painter()
                    .rect_filled(full_rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
            }

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(full_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child_ui.add_space(TREE_LABEL_HOFFSET);

            let prefix_string = indent_prefix(ctx.depth);
            child_ui.add(
                egui::Label::new(egui::RichText::new(prefix_string).color(text_color))
                    .selectable(false),
            );

            child_ui.allocate_response(
                egui::vec2(crate::icon::IconSize::Small.to_vec2().x, 0.0),
                egui::Sense::hover(),
            );

            if entry.is_markdown() {
                child_ui.add(
                    crate::icon::Icon::Document
                        .image(crate::icon::IconSize::Medium)
                        .tint(text_color),
                );
            } else {
                child_ui.allocate_response(
                    egui::vec2(crate::icon::IconSize::Medium.to_vec2().x, 0.0),
                    egui::Sense::hover(),
                );
            };

            let mut rich = egui::RichText::new(name).color(text_color);
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

