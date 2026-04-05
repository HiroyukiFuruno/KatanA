use super::types::*;
use crate::app::action::ActionOps;
use crate::app_state::{AppAction, ViewMode};
use crate::preview_pane::DownloadRequest;
use crate::shell::KatanaApp;
use crate::shell_logic::ShellLogicOps;
use crate::theme_bridge;
use eframe::egui;

const CHEVRON_ICON_SIZE: f32 = 10.0;
const ACTIVITY_RAIL_PADDING: f32 = 8.0;

impl<'a> MainPanels<'a> {
    pub fn new(
        app: &'a mut KatanaApp,
        theme_colors: &'a katana_platform::theme::ThemeColors,
    ) -> Self {
        Self { app, theme_colors }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let app = self.app;
        let theme_colors = self.theme_colors;
        let export_filenames: Vec<String> = app
            .export_tasks
            .iter()
            .map(|t| t.filename.clone())
            .collect();
        let mut resolved_status = app.state.layout.status_message.as_ref();
        let settings_err_tuple;
        if let Some(err) = &app.state.config.settings_save_error {
            settings_err_tuple = (err.clone(), crate::app_state::StatusType::Error);
            resolved_status = Some(&settings_err_tuple);
        }

        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            let action = crate::views::top_bar::StatusBar::new(
                resolved_status,
                app.state.is_dirty(),
                &export_filenames,
            )
            .show(ui, app.state.diagnostics.total_problems());
            if let Some(a) = action {
                app.pending_action = a;
            }
        });

        crate::views::panels::problems::ProblemsPanel::new(&mut app.state, &mut app.pending_action)
            .show(ui);

        WindowTitle::new(app).show(ui);

        TitleBar::new(app, theme_colors).show(ui);

        WorkspaceSidebar::new(app).show(ui);

        TabToolbar::new(app).show(ui);

        CentralContent::new(app).show(ui)
    }
}

impl<'a> WindowTitle<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        let ws_root_for_title = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
        let title_text = match app.state.active_document() {
            Some(doc) => {
                let fname = doc.file_name().unwrap_or("");
                let rel =
                    ShellLogicOps::relative_full_path(&doc.path, ws_root_for_title.as_deref());
                crate::shell_logic::ShellLogicOps::format_window_title(
                    fname,
                    &rel,
                    &crate::i18n::I18nOps::get().menu.release_notes,
                )
            }
            None => "KatanA".to_string(),
        };
        if app.state.layout.last_window_title != title_text {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));
            app.state.layout.last_window_title = title_text;
        }
    }
}

impl<'a> TitleBar<'a> {
    pub(crate) fn new(
        app: &'a KatanaApp,
        theme_colors: &'a katana_platform::theme::ThemeColors,
    ) -> Self {
        Self { app, theme_colors }
    }
    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        let theme_colors = self.theme_colors;
        let title_text = &app.state.layout.last_window_title;
        egui::Panel::top("app_title_bar").show_inside(ui, |ui| {
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.centered_and_justified(|ui| {
                        let title_color = theme_bridge::ThemeBridgeOps::rgb_to_color32(
                            theme_colors.system.title_bar_text,
                        );
                        ui.label(egui::RichText::new(title_text).small().color(title_color));
                    });
                })
                .show(ui);
        });
    }
}

impl<'a> WorkspaceSidebar<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;

        egui::Panel::left("activity_rail")
            .resizable(false)
            .exact_size(crate::shell::SIDEBAR_COLLAPSED_TOGGLE_WIDTH + ACTIVITY_RAIL_PADDING)
            .frame(
                egui::Frame::side_top_panel(&ui.ctx().global_style())
                    .inner_margin(egui::Margin::ZERO)

            )
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ACTIVITY_RAIL_PADDING);

                    let order = app.state.config.settings.settings().layout.activity_rail_order.clone();
                    let recent_paths = app.state.config.settings.settings().workspace.paths.clone();

                    let compute_drop_points_y = |rects: &[(usize, egui::Rect)]| -> Vec<(usize, f32)> {
                        let mut points = Vec::new();
                        if rects.is_empty() { return points; }
                        for i in 0..rects.len() {
                            if i == 0 {
                                points.push((0, rects[i].1.top()));
                            } else {
                                points.push((i, (rects[i - 1].1.bottom() + rects[i].1.top()) / 2.0));
                            }
                        }
                        points.push((rects.len(), rects.last().unwrap().1.bottom()));
                        points
                    };

                    let resolve_drag_drop_y = |src_idx: usize, ghost_y: f32, rects: &[(usize, egui::Rect)]| -> Option<crate::app_state::AppAction> {
                        let points = compute_drop_points_y(rects);
                        let mut best_dist = f32::MAX;
                        let mut best_to = None;
                        for (insert_idx, y) in points {
                            let dist = (ghost_y - y).abs();
                            if dist < best_dist {
                                best_dist = dist;
                                best_to = Some(insert_idx);
                            }
                        }
                        if let Some(to) = best_to
                            && src_idx != to && src_idx + 1 != to {
                                return Some(crate::app_state::AppAction::ReorderActivityRail { from: src_idx, to });
                            }
                        None
                    };

                    let mut rail_rects = Vec::new();
                    let mut responses = Vec::new();
                    let mut dragged_source: Option<(usize, f32)> = None;
                    let mut reorder_action = None;
                    let mut current_hovered_drop_y = None;

                    for (idx, item) in order.iter().enumerate() {
                        let interact_id = egui::Id::new("rail_drag").with(idx);
                        let is_being_dragged = ui.ctx().is_being_dragged(interact_id);

                        let act_resp = if is_being_dragged {
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), crate::icon::IconSize::Large.to_vec2().y + ACTIVITY_RAIL_PADDING),
                                egui::Sense::hover(),
                            );
                            Some(ui.interact(rect, interact_id, egui::Sense::click_and_drag()))
                        } else {
                            match item {
                                katana_platform::settings::ActivityRailItem::WorkspaceToggle => {
                                    let ws_icon = if app.state.layout.show_workspace {
                                        crate::Icon::FolderOpen
                                    } else {
                                        crate::Icon::FolderClosed
                                    };
                                    /* WHY: allow(icon_button_fill) */
                                    let mut btn = egui::Button::image(ws_icon.ui_image(ui, crate::icon::IconSize::Large)).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() })
                                        .sense(egui::Sense::hover());
                                    if app.state.layout.show_workspace {
                                        btn = btn.fill(ui.visuals().selection.bg_fill);
                                    }
                                    let resp = ui.add(btn);
                                    let interact_resp = ui
                                        .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
                                        .on_hover_text(crate::i18n::I18nOps::get().workspace.workspace_title.clone());
                                    if interact_resp.clicked() {
                                        app.pending_action = crate::app_state::AppAction::ToggleWorkspace;
                                    }
                                    Some(interact_resp)
                                }
                                katana_platform::settings::ActivityRailItem::Search => {
                                    /* WHY: allow(icon_button_fill) */
                                    let mut btn = egui::Button::image(
                                        crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Large)
                                    ).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() })
                                    .sense(egui::Sense::hover());
                                    if app.state.layout.show_search_modal {
                                        btn = btn.fill(ui.visuals().selection.bg_fill);
                                    }
                                    let resp = ui.add(btn);
                                    let interact_resp = ui
                                        .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
                                        .on_hover_text(crate::i18n::I18nOps::get().search.modal_title.clone());
                                    interact_resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, crate::i18n::I18nOps::get().search.modal_title.clone()));
                                    if interact_resp.clicked() {
                                        app.pending_action = crate::app_state::AppAction::ToggleSearchModal;
                                    }
                                    Some(interact_resp)
                                }
                                katana_platform::settings::ActivityRailItem::Settings => {
                                    /* WHY: allow(icon_button_fill) */
                                    let mut btn = egui::Button::image(
                                        crate::Icon::Settings.ui_image(ui, crate::icon::IconSize::Large)
                                    ).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() })
                                    .sense(egui::Sense::hover());
                                    if app.state.layout.show_settings {
                                        btn = btn.fill(ui.visuals().selection.bg_fill);
                                    }
                                    let resp = ui.add(btn);
                                    let interact_resp = ui
                                        .interact(resp.rect, interact_id, egui::Sense::click_and_drag())
                                        .on_hover_text(crate::i18n::I18nOps::get().settings.title.clone());
                                    if interact_resp.clicked() {
                                        app.pending_action = crate::app_state::AppAction::ToggleSettings;
                                    }
                                    Some(interact_resp)
                                }
                                katana_platform::settings::ActivityRailItem::History => {
                                    let history_menu_id = egui::Id::new("history_menu").with(idx);
                                    let is_open = ui
                                        .data(|data| data.get_temp::<bool>(history_menu_id).unwrap_or(false));

                                    /* WHY: allow(icon_button_fill) */
                                    let mut btn = egui::Button::image(
                                        crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Large)
                                    ).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() })
                                    .sense(egui::Sense::hover());
                                    if is_open {
                                        btn = btn.fill(ui.visuals().selection.bg_fill);
                                    }
                                    let resp = ui.add_enabled(!recent_paths.is_empty(), btn);
                                    let hover_text = crate::i18n::I18nOps::get().workspace.recent_workspaces.clone();

                                    let interact_resp =
                                        ui.interact(resp.rect, interact_id, egui::Sense::click_and_drag());
                                    let interact_resp = if recent_paths.is_empty() {
                                        interact_resp.on_disabled_hover_text(hover_text)
                                    } else {
                                        interact_resp.on_hover_text(hover_text)
                                    };

                                    if interact_resp.clicked() && !recent_paths.is_empty() {
                                        ui.data_mut(|data| data.insert_temp(history_menu_id, !is_open));
                                    }

                                    if is_open {
                                        const HISTORY_MENU_X_OFFSET: f32 = 4.0;
                                        const HISTORY_MENU_MAX_WIDTH: f32 = 320.0;

                                        let popup_area = egui::Area::new(egui::Id::new("history_menu_area").with(idx))
                                            .order(egui::Order::Foreground)
                                            .fixed_pos(interact_resp.rect.right_top() + egui::vec2(HISTORY_MENU_X_OFFSET, 0.0))
                                            .show(ui.ctx(), |ui| {
                                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                    ui.set_max_width(HISTORY_MENU_MAX_WIDTH);
                                                    for path in recent_paths.iter().rev() {
                                                        crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                                                            if ui
                                                                /* WHY: allow(icon_button_fill) */
                                                                .add(egui::Button::image(
                                                                    crate::Icon::Remove.ui_image(ui, crate::icon::IconSize::Small),
                                                                ).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() }))
                                                                .on_hover_text(
                                                                    crate::i18n::I18nOps::get().action.remove_workspace.clone(),
                                                                )
                                                                .clicked()
                                                            {
                                                                app.pending_action =
                                                                    crate::app_state::AppAction::RemoveWorkspace(
                                                                        path.clone(),
                                                                    );
                                                                ui.data_mut(|data| {
                                                                    data.insert_temp(history_menu_id, false)
                                                                });
                                                            }
                                                            if ui
                                                                .add(
                                                                    egui::Button::new(path)
                                                                        .frame_when_inactive(
                                                                            true,
                                                                        ),
                                                                )
                                                                .clicked()
                                                            {
                                                                app.pending_action =
                                                                    crate::app_state::AppAction::OpenWorkspace(
                                                                        std::path::PathBuf::from(path),
                                                                    );
                                                                ui.data_mut(|data| {
                                                                    data.insert_temp(history_menu_id, false)
                                                                });
                                                            }
                                                        }).show(ui);
                                                    }
                                                });
                                            });

                                        let clicked_elsewhere = ui.input(|i| {
                                            if i.pointer.any_click() {
                                                if let Some(pos) = i.pointer.interact_pos() {
                                                    !popup_area.response.rect.contains(pos)
                                                        && !interact_resp.rect.contains(pos)
                                                } else {
                                                    true
                                                }
                                            } else {
                                                false
                                            }
                                        });

                                        if clicked_elsewhere || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                            ui.data_mut(|data| data.insert_temp(history_menu_id, false));
                                        }
                                    }
                                    Some(interact_resp)
                                }
                            }
                        };

                        if let Some(interact_resp) = act_resp {
                            rail_rects.push((idx, interact_resp.rect));
                            responses.push((idx, item, interact_id, is_being_dragged, interact_resp));
                        }

                        ui.add_space(ACTIVITY_RAIL_PADDING);
                    }

                    for (idx, item, _interact_id, is_being_dragged, interact_resp) in responses {
                        if (interact_resp.drag_started() || is_being_dragged)
                            && let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                let press_origin = ui.input(|i| i.pointer.press_origin()).unwrap_or(pointer_pos);
                                let drag_offset = pointer_pos - press_origin;
                                let ghost_rect = interact_resp.rect.translate(drag_offset);

                                ui.memory_mut(|mem| {
                                    mem.data.insert_temp(
                                        egui::Id::new("drag_ghost_y").with(idx),
                                        ghost_rect.center().y,
                                    )
                                });

                                egui::Area::new(egui::Id::new("rail_ghost").with(idx))
                                    .fixed_pos(ghost_rect.min)
                                    .order(egui::Order::Tooltip)
                                    .show(ui.ctx(), |ui| {
                                        match item {
                                            katana_platform::settings::ActivityRailItem::WorkspaceToggle => {
                                                let icon = if app.state.layout.show_workspace { crate::Icon::FolderOpen } else { crate::Icon::FolderClosed };
                                                /* WHY: allow(icon_button_fill) */
                                                let mut b = egui::Button::image(icon.ui_image(ui, crate::icon::IconSize::Large)).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() });
                                                if app.state.layout.show_workspace { b = b.fill(ui.visuals().selection.bg_fill); }
                                                ui.add(b);
                                            }
                                            katana_platform::settings::ActivityRailItem::Search => {
                                                /* WHY: allow(icon_button_fill) */
                                                let mut b = egui::Button::image(crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Large)).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() });
                                                if app.state.layout.show_search_modal { b = b.fill(ui.visuals().selection.bg_fill); }
                                                let r = ui.add(b);
                                                r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Search"));
                                            }
                                            katana_platform::settings::ActivityRailItem::Settings => {
                                                /* WHY: allow(icon_button_fill) */
                                                let mut b = egui::Button::image(crate::Icon::Settings.ui_image(ui, crate::icon::IconSize::Large)).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() });
                                                if app.state.layout.show_settings { b = b.fill(ui.visuals().selection.bg_fill); }
                                                ui.add(b);
                                            }
                                            katana_platform::settings::ActivityRailItem::History => {
                                                /* WHY: allow(icon_button_fill) */
                                                ui.add(egui::Button::image(crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Large)).fill(if ui.visuals().dark_mode { crate::theme_bridge::TRANSPARENT } else { crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg() }));
                                            }
                                        }
                                    });
                            }

                        if interact_resp.drag_stopped()
                            && let Some(ghost_y) = ui.memory(|mem| mem.data.get_temp::<f32>(egui::Id::new("drag_ghost_y").with(idx))) {
                                dragged_source = Some((idx, ghost_y));
                            }

                        if is_being_dragged
                            && let Some(ghost_y) = ui.memory(|mem| mem.data.get_temp::<f32>(egui::Id::new("drag_ghost_y").with(idx))) {
                                let drop_points = compute_drop_points_y(&rail_rects);
                                let mut best_dist = f32::MAX;
                                let mut best_y = None;
                                for (_insert_idx, y) in drop_points {
                                    let dist = (ghost_y - y).abs();
                                    if dist < best_dist {
                                        best_dist = dist;
                                        best_y = Some(y);
                                    }
                                }
                                if let Some(y) = best_y {
                                    current_hovered_drop_y = Some((y, rail_rects[idx].1.x_range()));
                                }
                            }
                    }

                    if let Some((target_y, x_range)) = current_hovered_drop_y {
                        let indicator_id = egui::Id::new("rail_drop_indicator");
                        let animated_y = ui.ctx().animate_value_with_time(
                            indicator_id,
                            target_y,
                            crate::shell::TAB_DROP_ANIMATION_TIME,
                        );
                        let stroke = egui::Stroke::new(
                            crate::shell::TAB_DROP_INDICATOR_WIDTH,
                            ui.visuals().selection.bg_fill,
                        );
                        ui.painter().hline(x_range, animated_y, stroke);
                    }

                    if let Some((src_idx, ghost_center_y)) = dragged_source
                        && let Some(action) = resolve_drag_drop_y(src_idx, ghost_center_y, &rail_rects) {
                            reorder_action = Some(action);
                        }

                    if let Some(act) = reorder_action {
                        app.pending_action = act;
                    }
                });
            });

        if app.state.layout.show_workspace {
            egui::Panel::left("workspace_tree")
                .resizable(true)
                .min_size(crate::shell::FILE_TREE_PANEL_MIN_WIDTH)
                .default_size(crate::shell::FILE_TREE_PANEL_DEFAULT_WIDTH)
                .show_inside(ui, |ui| {
                    let active_path = app.state.active_path().map(|p| p.to_path_buf());
                    crate::views::panels::workspace::WorkspacePanel::new(
                        &mut app.state.workspace,
                        &mut app.state.search,
                        &app.state.config.settings.settings().workspace.paths,
                        active_path.as_deref(),
                        &mut app.pending_action,
                    )
                    .show(ui);
                });
        }
    }
}

impl<'a> TabToolbar<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = self.app;
        egui::Panel::top("tab_toolbar").show_inside(ui, |ui| {
            let ws_root = app
                .state
                .workspace
                .data
                .as_ref()
                .map(|ws| ws.root.as_path());
            let tab_action = crate::views::top_bar::TabBar::new(
                ws_root,
                &app.state.document.open_documents,
                app.state.document.active_doc_idx,
                &app.state.document.recently_closed_tabs,
                &app.state.document.tab_groups,
                &app.state.layout.inline_rename_group,
            )
            .show(ui);
            if let Some(a) = tab_action {
                app.pending_action = a;
            }
            let doc_info = app.state.active_document().map(|doc| {
                let d_path = doc.path.to_string_lossy();
                let is_changelog = d_path.starts_with("Katana://ChangeLog");
                (doc.path.clone(), is_changelog)
            });

            if let Some((doc_path, is_changelog)) = doc_info {
                let mut out_action = None;
                let bar_height = ui.spacing().interact_size.y;
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), bar_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if !is_changelog {
                            let ws_root =
                                app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
                            let rel =
                                ShellLogicOps::relative_full_path(&doc_path, ws_root.as_deref());
                            let breadcrumb_action =
                                Breadcrumbs::new(app, &rel, ws_root.as_deref()).show(ui);
                            if let Some(a) = breadcrumb_action {
                                out_action = Some(a);
                            }
                        }

                        let view_action = crate::views::top_bar::ViewModeBar::new(
                            app.state.active_view_mode(),
                            is_changelog,
                            app.state.active_split_direction(),
                            app.state.active_pane_order(),
                            app.state
                                .config
                                .settings
                                .settings()
                                .behavior
                                .scroll_sync_enabled,
                            app.state.scroll.sync_override,
                            app.state.update.available.is_some(),
                            app.state.update.checking,
                            true,
                        )
                        .show(ui, &mut app.state.search);
                        if let Some(a) = view_action {
                            out_action = Some(a);
                        }
                    },
                );

                if app.state.search.doc_search_open {
                    let search_action =
                        crate::views::top_bar::DocSearchBar::show(ui, &mut app.state.search);
                    if let Some(a) = search_action {
                        out_action = Some(a);
                    }
                }

                if let Some(a) = out_action {
                    app.pending_action = a;
                }
            }
        });
    }
}

impl<'a> Breadcrumbs<'a> {
    pub(crate) fn new(
        app: &'a KatanaApp,
        rel: &'a str,
        ws_root: Option<&'a std::path::Path>,
    ) -> Self {
        Self { app, rel, ws_root }
    }
    pub(crate) fn show(self, ui: &mut egui::Ui) -> Option<AppAction> {
        let app = self.app;
        let rel = self.rel;
        let ws_root = self.ws_root;
        let mut breadcrumb_action = None;
        ui.horizontal_centered(|ui| {
            let segments: Vec<&str> = rel.split('/').collect();
            let mut current_path = ws_root.map(std::path::PathBuf::from).unwrap_or_default();
            for (i, seg) in segments.iter().enumerate() {
                if i > 0 {
                    ui.add(
                        egui::Image::new(crate::Icon::ChevronRight.uri())
                            .tint(ui.visuals().text_color())
                            .fit_to_exact_size(egui::vec2(CHEVRON_ICON_SIZE, CHEVRON_ICON_SIZE)),
                    );
                }

                if ws_root.is_none() {
                    ui.label(egui::RichText::new(*seg).small());
                    continue;
                }

                current_path = current_path.join(seg);
                let is_last = i == segments.len() - 1;

                if is_last {
                    ui.add(
                        egui::Label::new(egui::RichText::new(*seg).small())
                            .sense(egui::Sense::hover()),
                    );
                } else {
                    /* WHY: allow(conditional_frame) in popup/list context; future: standardize as atom */
                    ui.menu_button(egui::RichText::new(*seg).small(), |ui| {
                        let mut ctx_action = crate::app_state::AppAction::None;

                        if let Some(ws) = &app.state.workspace.data
                            && let Some(katana_core::workspace::TreeEntry::Directory {
                                children,
                                ..
                            }) = crate::views::panels::tree::TreeLogicOps::find_node_in_tree(
                                &ws.tree,
                                &current_path,
                            )
                        {
                            crate::views::panels::workspace::BreadcrumbMenu::new(
                                children,
                                &mut ctx_action,
                            )
                            .show(ui);
                        }

                        if !matches!(ctx_action, crate::app_state::AppAction::None) {
                            breadcrumb_action = Some(ctx_action);
                            ui.close();
                        }
                    });
                }
            }
        });
        breadcrumb_action
    }
}

impl<'a> CentralContent<'a> {
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    pub(crate) fn show(self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let app = self.app;
        let mut download_req: Option<DownloadRequest> = None;
        let current_mode = app.state.active_view_mode();
        let is_split = current_mode == ViewMode::Split;
        let mut is_changelog_tab = false;

        if let Some(doc) = app.state.active_document()
            && doc.path.to_string_lossy().starts_with("Katana://ChangeLog")
        {
            is_changelog_tab = true;
        }

        if app.state.layout.show_toc
            && app.state.config.settings.settings().layout.toc_visible
            && let Some(doc) = app.state.active_document()
            && let Some(preview) = app.tab_previews.iter_mut().find(|p| p.path == doc.path)
        {
            crate::views::panels::toc::TocPanel::new(&mut preview.pane, &app.state).show(ui);
        }

        if is_changelog_tab {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                crate::changelog::ChangelogOps::render_release_notes_tab(
                    ui,
                    &app.changelog_sections,
                    app.changelog_rx.is_some(),
                );
            });
        } else {
            /* WHY: Search logic is now inline in ViewModeBar */

            if is_split {
                let split_dir = app.state.active_split_direction();
                let pane_order = app.state.active_pane_order();
                let ctx = ui.ctx().clone();
                download_req =
                    crate::views::layout::split::SplitMode::new(&ctx, app, split_dir, pane_order)
                        .show(ui);
            }

            if !is_split {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(0.0))
                    .show_inside(ui, |ui| match current_mode {
                        ViewMode::CodeOnly => {
                            crate::views::panels::editor::EditorContent::new(
                                app.state.document.active_document(),
                                &mut app.state.scroll,
                                &mut app.pending_action,
                                false,
                                &app.state.search.doc_search_matches,
                                app.state.search.doc_search_active_index,
                            )
                            .show(ui);
                        }
                        ViewMode::PreviewOnly => {
                            crate::views::layout::split::PreviewOnly::new(ui, app).show();
                        }
                        ViewMode::Split => {}
                    });
            }
        }

        download_req
    }
}

impl AppFrameOps {
    pub(crate) fn intercept_url_commands(ctx: &egui::Context, app: &mut KatanaApp) {
        let commands = ctx.output_mut(|o| std::mem::take(&mut o.commands));
        let mut unprocessed_commands = Vec::new();

        for cmd in commands {
            if let egui::OutputCommand::OpenUrl(open) = &cmd {
                let url = &open.url;
                if url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("mailto:")
                {
                    unprocessed_commands.push(cmd);
                } else {
                    let mut path = std::path::PathBuf::from(url);
                    if path.is_relative()
                        && let Some(doc) = app.state.active_document()
                        && let Some(parent) = doc.path.parent()
                    {
                        path = parent.join(path);
                    }
                    app.process_action(ctx, AppAction::SelectDocument(path));
                }
            } else {
                unprocessed_commands.push(cmd);
            }
        }
        ctx.output_mut(|o| o.commands.extend(unprocessed_commands));
    }
}
