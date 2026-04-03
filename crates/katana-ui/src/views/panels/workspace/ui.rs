use crate::app_state::AppAction;
use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING,
    RECENT_WORKSPACES_SPACING, TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT,
};
use crate::shell_ui::{
    LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, TreeRenderContext,
    WORKSPACE_SPINNER_INNER_MARGIN, WORKSPACE_SPINNER_OUTER_MARGIN, WORKSPACE_SPINNER_TEXT_MARGIN,
};
use eframe::egui;

use super::logic::{update_search_filter_cache, update_tree_expansion};

// --- BreadcrumbMenu ---
pub(crate) struct BreadcrumbMenu<'a> {
    pub entries: &'a [katana_core::workspace::TreeEntry],
    pub action: &'a mut crate::app_state::AppAction,
}

impl<'a> BreadcrumbMenu<'a> {
    pub fn new(
        entries: &'a [katana_core::workspace::TreeEntry],
        action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self { entries, action }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let entries = self.entries;
        let action = self.action;
        for entry in entries {
            match entry {
                katana_core::workspace::TreeEntry::Directory { path, children } => {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    ui.menu_button(name, |ui| {
                        BreadcrumbMenu::new(children, action).show(ui);
                    });
                }
                katana_core::workspace::TreeEntry::File { path } => {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    if ui.button(name).clicked() {
                        *action = crate::app_state::AppAction::SelectDocument(path.clone());
                        ui.close();
                    }
                }
            }
        }
    }
}

// --- FileEntryNode ---
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
                ui.painter()
                    .rect_filled(full_rect, 2.0, ui.visuals().widgets.hovered.bg_fill);
            }

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(full_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            child_ui.add_space(TREE_LABEL_HOFFSET);

            let prefix_string = crate::shell_ui::ShellUiOps::indent_prefix(ctx.depth);
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

// --- DirectoryEntryNode ---
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
            let prefix = crate::shell_ui::ShellUiOps::indent_prefix(ctx.depth);
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

// --- TreeEntryNode ---
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
        if let Some(fs) = ctx.filter_set
            && !fs.contains(entry_path)
        {
            return;
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

// --- WorkspaceContent ---
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
        let workspace = self.workspace;
        let search = self.search;
        let recent_paths = self.recent_paths;
        let active_path = self.active_path;
        let action = self.action;

        if let Some(ws) = &workspace.data {
            let entries = ws.tree.clone();
            let ws_root = ws.root.clone();
            let _ = ws;

            update_tree_expansion(workspace);
            update_search_filter_cache(search, &ws_root, &entries);

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
                        let mut flat_entries = Vec::new();
                        fn collect_files<'a>(
                            entries: &'a [katana_core::workspace::TreeEntry],
                            out: &mut Vec<&'a katana_core::workspace::TreeEntry>,
                        ) {
                            for e in entries {
                                match e {
                                    katana_core::workspace::TreeEntry::File { .. } => out.push(e),
                                    katana_core::workspace::TreeEntry::Directory {
                                        children,
                                        ..
                                    } => collect_files(children, out),
                                }
                            }
                        }
                        collect_files(&entries, &mut flat_entries);

                        for entry in flat_entries {
                            if let katana_core::workspace::TreeEntry::File { path } = entry {
                                FileEntryNode::new(entry, path, &mut ctx).show(ui);
                            }
                        }
                    } else {
                        for entry in &entries {
                            TreeEntryNode::new(entry, &mut ctx).show(ui);
                        }
                    }
                });
        } else {
            ui.label(crate::i18n::get().workspace.no_workspace_open.clone());
            ui.add_space(NO_WORKSPACE_BOTTOM_SPACING);
            if ui
                .button(crate::i18n::get().menu.open_workspace.clone())
                .clicked()
                && let Some(path) = crate::shell_ui::ShellUiOps::open_folder_dialog()
            {
                *action = AppAction::OpenWorkspace(path);
            }

            if !recent_paths.is_empty() {
                ui.add_space(RECENT_WORKSPACES_SPACING);
                ui.separator();
                ui.add_space(RECENT_WORKSPACES_SPACING);
                ui.heading(crate::i18n::get().workspace.recent_workspaces.clone());
                ui.add_space(RECENT_WORKSPACES_ITEM_SPACING);
                for path in recent_paths.iter().rev() {
                    ui.horizontal(|ui| {
                        if ui
                            .button("×")
                            .on_hover_text(crate::i18n::get().action.remove_workspace.clone())
                            .clicked()
                        {
                            *action = AppAction::RemoveWorkspace(path.clone());
                        }
                        if ui.selectable_label(false, path).clicked() {
                            *action = AppAction::OpenWorkspace(std::path::PathBuf::from(path));
                        }
                    });
                }
            }
        }
    }
}

// --- WorkspaceHeader ---
pub(crate) struct WorkspaceHeader<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceHeader<'a> {
    pub fn new(
        workspace: &'a mut crate::app_state::WorkspaceState,
        search: &'a mut crate::app_state::SearchState,
        recent_paths: &'a [String],
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            workspace,
            search,
            recent_paths,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let (workspace, search, _recent_paths, action) =
            (self.workspace, self.search, self.recent_paths, self.action);

        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::from_gray(LIGHT_MODE_ICON_BG)
        };

        if workspace.data.is_some() {
            let ws_root = workspace
                .data
                .as_ref()
                .map(|w| w.root.clone())
                .unwrap_or_default();
            let is_flat = workspace.is_flat_view(&ws_root);

            ui.horizontal(|ui| {
                ui.add_enabled_ui(!is_flat, |ui| {
                    let btn_resp = ui
                        .add(egui::Button::image(
                            crate::Icon::ExpandAll.ui_image(ui, crate::icon::IconSize::Small),
                        ))
                        .on_hover_text(crate::i18n::get().action.expand_all.clone());
                    btn_resp.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "+")
                    });
                    if btn_resp.clicked()
                        && let Some(ws) = &workspace.data
                    {
                        workspace
                            .expanded_directories
                            .extend(ws.collect_all_directory_paths());
                    }

                    let btn_resp = ui
                        .add(egui::Button::image(
                            crate::Icon::CollapseAll.ui_image(ui, crate::icon::IconSize::Small),
                        ))
                        .on_hover_text(crate::i18n::get().action.collapse_all.clone());
                    btn_resp.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "-")
                    });
                    if btn_resp.clicked() {
                        workspace.force_tree_open = Some(false);
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("...", |ui| {
                        let flat_label = format!(
                            "{} {}",
                            if is_flat { "✔" } else { "  " },
                            crate::i18n::get().workspace.flat_view
                        );

                        if ui.button(flat_label).clicked() {
                            workspace.set_flat_view(ws_root, !is_flat);
                            ui.close();
                        }
                    });

                    if ui
                        .add(
                            egui::Button::image_and_text(
                                crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Small),
                                crate::shell_ui::ShellUiOps::invisible_label("Refresh"),
                            )
                            .fill(icon_bg),
                        )
                        .on_hover_text(crate::i18n::get().action.refresh_workspace.clone())
                        .clicked()
                    {
                        *action = AppAction::RefreshWorkspace;
                    }

                    let filter_btn_color = if search.filter_enabled {
                        if ui.visuals().dark_mode {
                            ui.visuals().selection.bg_fill
                        } else {
                            crate::theme_bridge::from_gray(LIGHT_MODE_ICON_ACTIVE_BG)
                        }
                    } else {
                        icon_bg
                    };

                    if ui
                        .add(
                            egui::Button::image_and_text(
                                crate::Icon::Filter.ui_image(ui, crate::icon::IconSize::Small),
                                crate::shell_ui::ShellUiOps::invisible_label("∇"),
                            )
                            .fill(filter_btn_color),
                        )
                        .on_hover_text(crate::i18n::get().action.toggle_filter.clone())
                        .clicked()
                    {
                        *action = AppAction::ToggleWorkspaceFilter;
                    }
                });
            });

            if search.filter_enabled {
                let mut is_valid_regex = true;
                if !search.filter_query.is_empty() {
                    is_valid_regex = regex::Regex::new(&search.filter_query).is_ok();
                }
                ui.horizontal(|ui| {
                    let text_color = if is_valid_regex {
                        ui.visuals().text_color()
                    } else {
                        ui.ctx()
                            .data(|d| {
                                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                    "katana_theme_colors",
                                ))
                            })
                            .map_or(crate::theme_bridge::WHITE, |tc| {
                                crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                            })
                    };
                    ui.add(
                        egui::TextEdit::singleline(&mut search.filter_query)
                            .text_color(text_color)
                            .hint_text(&crate::i18n::get().workspace.filter_regex_hint)
                            .desired_width(f32::INFINITY),
                    );
                });
            }
        }
    }
}

// --- WorkspacePanel ---
pub(crate) struct WorkspacePanel<'a> {
    pub workspace: &'a mut crate::app_state::WorkspaceState,
    pub search: &'a mut crate::app_state::SearchState,
    pub recent_paths: &'a [String],
    pub active_path: Option<&'a std::path::Path>,
    pub action: &'a mut AppAction,
}

impl<'a> WorkspacePanel<'a> {
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
        let workspace = self.workspace;
        let search = self.search;
        let recent_paths = self.recent_paths;
        let active_path = self.active_path;
        let action = self.action;
        let panel_width = ui.available_width();
        ui.set_max_width(panel_width);
        ui.set_min_width(panel_width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

        let is_loading = workspace.is_loading;

        WorkspaceHeader::new(workspace, search, recent_paths, action).show(ui);

        ui.separator();

        if is_loading {
            ui.add_space(WORKSPACE_SPINNER_OUTER_MARGIN);
            ui.horizontal(|ui| {
                ui.add_space(WORKSPACE_SPINNER_INNER_MARGIN);
                ui.spinner();
                ui.add_space(WORKSPACE_SPINNER_TEXT_MARGIN);
                ui.label(crate::i18n::get().action.refresh_workspace.clone());
            });
        } else {
            WorkspaceContent::new(workspace, search, recent_paths, active_path, action).show(ui);
        }
    }
}
