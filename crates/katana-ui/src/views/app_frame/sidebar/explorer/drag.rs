use crate::shell::KatanaApp;
use crate::views::app_frame::types::*;
use eframe::egui;

impl ExplorerSidebarDrag {
    pub(crate) fn compute_drop_points_y(rects: &[(usize, egui::Rect)]) -> Vec<(usize, f32)> {
        let mut points = Vec::new();
        if rects.is_empty() {
            return points;
        }
        for i in 0..rects.len() {
            if i == 0 {
                points.push((0, rects[i].1.top()));
            } else {
                points.push((i, (rects[i - 1].1.bottom() + rects[i].1.top()) / 2.0));
            }
        }
        points.push((rects.len(), rects.last().unwrap().1.bottom()));
        points
    }

    pub(crate) fn resolve_drag_drop_y(
        src_idx: usize,
        ghost_y: f32,
        rects: &[(usize, egui::Rect)],
    ) -> Option<crate::app_state::AppAction> {
        let points = Self::compute_drop_points_y(rects);
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
            && src_idx != to
            && src_idx + 1 != to
        {
            return Some(crate::app_state::AppAction::ReorderActivityRail { from: src_idx, to });
        }
        None
    }

    pub(crate) fn handle_rail_drag_effects(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        responses: Vec<(
            usize,
            &katana_platform::settings::ActivityRailItem,
            egui::Id,
            bool,
            egui::Response,
        )>,
        rail_rects: &[(usize, egui::Rect)],
        dragged_source: &mut Option<(usize, f32)>,
        current_hovered_drop_y: &mut Option<(f32, egui::Rangef)>,
    ) {
        for (idx, item, _interact_id, is_being_dragged, interact_resp) in responses {
            if interact_resp.drag_started() || is_being_dragged {
                Self::render_drag_ghost(ui, app, idx, item, interact_resp.rect);
            }

            if interact_resp.drag_stopped()
                && let Some(ghost_y) = ui.memory(|mem| {
                    mem.data
                        .get_temp::<f32>(egui::Id::new("drag_ghost_y").with(idx))
                })
            {
                *dragged_source = Some((idx, ghost_y));
            }

            if is_being_dragged {
                Self::calculate_hover_drop(ui, idx, rail_rects, current_hovered_drop_y);
            }
        }
    }

    fn calculate_hover_drop(
        ui: &mut egui::Ui,
        idx: usize,
        rail_rects: &[(usize, egui::Rect)],
        current_hovered_drop_y: &mut Option<(f32, egui::Rangef)>,
    ) {
        if let Some(ghost_y) = ui.memory(|mem| {
            mem.data
                .get_temp::<f32>(egui::Id::new("drag_ghost_y").with(idx))
        }) {
            let drop_points = Self::compute_drop_points_y(rail_rects);
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
                *current_hovered_drop_y = Some((y, rail_rects[idx].1.x_range()));
            }
        }
    }

    fn render_drag_ghost(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        idx: usize,
        item: &katana_platform::settings::ActivityRailItem,
        interact_rect: egui::Rect,
    ) {
        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let press_origin = ui
                .input(|i| i.pointer.press_origin())
                .unwrap_or(pointer_pos);
            let drag_offset = pointer_pos - press_origin;
            let ghost_rect = interact_rect.translate(drag_offset);

            ui.memory_mut(|mem| {
                mem.data.insert_temp(
                    egui::Id::new("drag_ghost_y").with(idx),
                    ghost_rect.center().y,
                )
            });

            egui::Area::new(egui::Id::new("rail_ghost").with(idx))
                .fixed_pos(ghost_rect.min)
                .order(egui::Order::Tooltip)
                .show(ui.ctx(), |ui| match item {
                    katana_platform::settings::ActivityRailItem::AddWorkspace => {
                        ui.add(crate::Icon::Plus.button(ui, crate::icon::IconSize::Large));
                    }
                    katana_platform::settings::ActivityRailItem::WorkspaceToggle => {
                        ui.add(crate::Icon::FolderClosed.selected_button(
                            ui,
                            crate::icon::IconSize::Large,
                            app.state.layout.show_workspace_panel,
                        ));
                    }
                    katana_platform::settings::ActivityRailItem::ExplorerToggle => {
                        let icon = crate::Icon::Explorer;
                        ui.add(icon.selected_button(
                            ui,
                            crate::icon::IconSize::Large,
                            app.state.layout.show_explorer,
                        ));
                    }
                    katana_platform::settings::ActivityRailItem::Search => {
                        let r = ui.add(crate::Icon::Search.selected_button(
                            ui,
                            crate::icon::IconSize::Large,
                            app.state.layout.show_search_modal,
                        ));
                        r.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Search")
                        });
                    }
                    katana_platform::settings::ActivityRailItem::Settings => {
                        ui.add(crate::Icon::Settings.selected_button(
                            ui,
                            crate::icon::IconSize::Large,
                            app.state.layout.show_settings,
                        ));
                    }
                    katana_platform::settings::ActivityRailItem::History => {
                        ui.add(crate::Icon::History.button(ui, crate::icon::IconSize::Large));
                    }
                });
        }
    }
}
