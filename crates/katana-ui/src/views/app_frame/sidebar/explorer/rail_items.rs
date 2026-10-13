use crate::shell::KatanaApp;
use crate::views::app_frame::types::*;
use eframe::egui;

impl<'a> ExplorerSidebar<'a> {
    /* WHY: Renders the core activity rail items with drag-reorder capabilities. */
    pub(super) fn render_rail_items(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let order = app
            .state
            .config
            .settings
            .settings()
            .layout
            .activity_rail_order
            .clone();

        let mut rail_rects = Vec::new();
        let mut responses = Vec::new();
        let mut dragged_source: Option<(usize, f32)> = None;
        let mut current_hovered_drop_y = None;

        for (idx, item) in order.iter().enumerate() {
            let interact_id = egui::Id::new("rail_drag").with(idx);
            let is_being_dragged = ui.ctx().is_being_dragged(interact_id);

            let act_resp = if is_being_dragged {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(
                        ui.available_width(),
                        crate::icon::IconSize::Large.to_vec2().y
                            + crate::shell::ACTIVITY_RAIL_PADDING,
                    ),
                    egui::Sense::hover(),
                );
                Some(ui.interact(rect, interact_id, egui::Sense::click_and_drag()))
            } else {
                Self::render_single_act_rail_item(ui, app, item, interact_id, idx)
            };

            if let Some(interact_resp) = act_resp {
                rail_rects.push((idx, interact_resp.rect));
                responses.push((idx, item, interact_id, is_being_dragged, interact_resp));
                ui.add_space(crate::shell::ACTIVITY_RAIL_PADDING);
            }
        }

        ExplorerSidebarDrag::handle_rail_drag_effects(
            ui,
            app,
            responses,
            &rail_rects,
            &mut dragged_source,
            &mut current_hovered_drop_y,
        );

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
            && let Some(action) =
                ExplorerSidebarDrag::resolve_drag_drop_y(src_idx, ghost_center_y, &rail_rects)
        {
            app.pending_action = action;
        }
    }

    /* WHY: Dispatches single item rendering based on activity rail configuration. */
    fn render_single_act_rail_item(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        item: &katana_platform::settings::ActivityRailItem,
        interact_id: egui::Id,
        _idx: usize,
    ) -> Option<egui::Response> {
        match item {
            katana_platform::settings::ActivityRailItem::AddWorkspace => {
                ExplorerSidebarItems::render_add_workspace(ui, app, interact_id)
            }
            katana_platform::settings::ActivityRailItem::WorkspaceToggle => {
                ExplorerSidebarItems::render_workspace_toggle(ui, app, interact_id)
            }
            katana_platform::settings::ActivityRailItem::ExplorerToggle => {
                let resp = ExplorerSidebarItems::render_explorer_toggle(ui, app, interact_id);
                /* WHY: Store the button rect so the hover overlay knows where to check containment */
                if let Some(ref r) = resp {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(egui::Id::new("explorer_btn_rect"), r.rect));
                }
                resp
            }
            katana_platform::settings::ActivityRailItem::Search => {
                ExplorerSidebarItems::render_search_toggle(ui, app, interact_id)
            }
            katana_platform::settings::ActivityRailItem::History => None,
        }
    }
}
