use crate::app_state::AppAction;
use crate::views::top_bar::tab_drop_indicator::TabDropIndicator;
use crate::views::top_bar::types::TopBarOps;
use crate::views::top_bar::workspace_tab_bar_detail::WorkspaceTabBarDetail;
use eframe::egui;

const WIDTH_FIT_TOLERANCE: f32 = 0.5;

pub(crate) struct WorkspaceTabBar<'a> {
    open_workspace_tabs: &'a [String],
    active_workspace: Option<&'a str>,
    scroll_to_workspace_tab: &'a mut Option<std::path::PathBuf>,
}

struct WorkspaceRowState<'a> {
    action: &'a mut Option<AppAction>,
    tab_rects: &'a mut Vec<(usize, egui::Rect)>,
    dragged_source: &'a mut Option<(usize, f32)>,
    ghost_info_acc: &'a mut Option<(egui::Rect, egui::Rangef)>,
}

impl<'a> WorkspaceTabBar<'a> {
    pub(crate) fn new(
        open_workspace_tabs: &'a [String],
        active_workspace: Option<&'a str>,
        scroll_to_workspace_tab: &'a mut Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            open_workspace_tabs,
            active_workspace,
            scroll_to_workspace_tab,
        }
    }

    pub(crate) fn show(mut self, ui: &mut egui::Ui) -> Option<AppAction> {
        let mut action = None;
        let available_width = ui.available_width();
        let plus_button_width = ui.spacing().interact_size.x;
        let mut tab_rects = Vec::new();
        let mut dragged_source = None;
        let mut ghost_info_acc = None;
        let content_width = WorkspaceTabBarDetail::content_width(
            available_width,
            self.open_workspace_tabs.len(),
            plus_button_width,
        );
        let tab_width = WorkspaceTabBarDetail::tab_width(
            content_width,
            self.open_workspace_tabs.len(),
            plus_button_width,
        );
        let row_height = WorkspaceTabBarDetail::row_height(ui);
        let mut row_state = WorkspaceRowState {
            action: &mut action,
            tab_rects: &mut tab_rects,
            dragged_source: &mut dragged_source,
            ghost_info_acc: &mut ghost_info_acc,
        };
        if content_width <= available_width + WIDTH_FIT_TOLERANCE {
            self.render_workspace_row(ui, tab_width, plus_button_width, row_height, &mut row_state);
        } else {
            egui::ScrollArea::horizontal()
                .max_height(row_height)
                .auto_shrink([false, true])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .id_salt("workspace_tab_scroll")
                .show(ui, |ui| {
                    ui.set_min_width(content_width);
                    self.render_workspace_row(
                        ui,
                        tab_width,
                        plus_button_width,
                        row_height,
                        &mut row_state,
                    );
                });
        }
        if let Some((from, ghost_center_x)) = dragged_source
            && let Some(action_from_drag) =
                Self::resolve_reorder_action(from, ghost_center_x, &tab_rects)
        {
            action = Some(action_from_drag);
        }
        action
    }

    fn render_workspace_row(
        &mut self,
        ui: &mut egui::Ui,
        tab_width: f32,
        plus_button_width: f32,
        row_height: f32,
        row_state: &mut WorkspaceRowState<'_>,
    ) {
        let row_origin = ui.cursor().min;
        self.render_workspace_tabs(ui, row_origin, tab_width, row_height, row_state);
        let plus_rect = egui::Rect::from_min_size(
            egui::pos2(
                row_origin.x + tab_width * self.open_workspace_tabs.len() as f32,
                row_origin.y,
            ),
            egui::vec2(plus_button_width, row_height),
        );
        WorkspaceTabBarDetail::render_plus_button(ui, plus_rect, row_state.action);
        TabDropIndicator {
            tab_rects: row_state.tab_rects,
            ghost_info: *row_state.ghost_info_acc,
            id_salt: "workspace_tab_drop_indicator",
        }
        .render(ui);
    }

    fn render_workspace_tabs(
        &mut self,
        ui: &mut egui::Ui,
        row_origin: egui::Pos2,
        tab_width: f32,
        row_height: f32,
        row_state: &mut WorkspaceRowState<'_>,
    ) {
        for (index, path) in self.open_workspace_tabs.iter().enumerate() {
            let is_active = self.active_workspace == Some(path.as_str());
            let should_scroll = self
                .scroll_to_workspace_tab
                .as_ref()
                .is_some_and(|target| target.to_string_lossy() == path.as_str());
            let tab_rect = egui::Rect::from_min_size(
                egui::pos2(row_origin.x + tab_width * index as f32, row_origin.y),
                egui::vec2(tab_width, row_height),
            );
            let response = WorkspaceTabBarDetail::render_workspace_tab(
                ui,
                path,
                index,
                is_active,
                tab_rect,
                row_state.action,
            );
            row_state.tab_rects.push((index, tab_rect));
            if let Some(ghost_info) =
                WorkspaceTabBarDetail::handle_drag(ui, index, path, is_active, tab_rect, &response)
            {
                *row_state.ghost_info_acc = Some(ghost_info);
            }
            if let Some(stopped) = WorkspaceTabBarDetail::check_drag_stopped(ui, index, &response) {
                *row_state.dragged_source = Some(stopped);
            }
            if should_scroll {
                response.scroll_to_me(Some(egui::Align::Center));
                *self.scroll_to_workspace_tab = None;
            }
        }
    }

    fn resolve_reorder_action(
        from: usize,
        ghost_center_x: f32,
        tab_rects: &[(usize, egui::Rect)],
    ) -> Option<AppAction> {
        let drop_points = TopBarOps::compute_drop_points(tab_rects);
        let to_visual = TopBarOps::find_best_drop_index(&drop_points, ghost_center_x)?;
        let to_physical = tab_rects
            .get(to_visual)
            .map(|(index, _)| *index)
            .unwrap_or(tab_rects.len());
        if from != to_physical && from + 1 != to_physical {
            return Some(AppAction::ReorderWorkspaceTab {
                from,
                to: to_physical,
            });
        }
        None
    }
}
