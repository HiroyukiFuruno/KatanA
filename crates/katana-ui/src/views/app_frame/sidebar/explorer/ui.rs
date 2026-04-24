use crate::views::app_frame::types::*;

use crate::shell::KatanaApp;
use eframe::egui;

/* WHY: Animation speed (seconds) for the hover overlay fade-in */
const EXPLORER_ANIM_SPEED: f32 = 0.15;

/* WHY: UI implementation for ExplorerSidebar. */
impl<'a> ExplorerSidebar<'a> {
    /* WHY: Factory method to bind the layout state to the UI structure. */
    pub(crate) fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }

    pub(crate) fn show(self, ui: &mut egui::Ui) {
        let app = &mut *self.app;

        let rail_resp = egui::Panel::left("activity_rail")
            .resizable(false)
            .exact_size(
                crate::shell::SIDEBAR_COLLAPSED_TOGGLE_WIDTH + crate::shell::ACTIVITY_RAIL_PADDING,
            )
            .frame(
                egui::Frame::side_top_panel(&ui.ctx().global_style())
                    .inner_margin(egui::Margin::ZERO),
            )
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(crate::shell::ACTIVITY_RAIL_PADDING);
                        let settings_id = egui::Id::new("rail_fixed").with("settings");
                        ExplorerSidebarItems::render_settings_toggle(ui, app, settings_id);

                        ui.add_space(crate::shell::ACTIVITY_RAIL_PADDING);
                        let history_id = egui::Id::new("rail_fixed").with("history");
                        ExplorerSidebarItems::render_history_toggle(ui, app, history_id, 0);

                        ui.add_space(crate::shell::ACTIVITY_RAIL_PADDING);
                        let help_id = egui::Id::new("rail_fixed").with("help");
                        ExplorerSidebarItems::render_help_toggle(ui, app, help_id);

                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(crate::shell::ACTIVITY_RAIL_PADDING);
                            Self::render_rail_items(ui, app);
                        });
                    });
                });
            });

        /* WHY: Defined before the Case 1/2 split so Case 1 can reset it.
         * Ensures hover does not immediately re-open after unpinning (spec item 4). */
        let hover_id = egui::Id::new("explorer_hover_open");

        /* Case 1: PINNED — SidePanel pushes content aside. */
        if app.state.layout.show_explorer {
            egui::Panel::left("explorer_tree")
                .resizable(true)
                .min_size(crate::shell::FILE_TREE_PANEL_MIN_WIDTH)
                .default_size(crate::shell::FILE_TREE_PANEL_DEFAULT_WIDTH)
                .show_inside(ui, |ui| {
                    let active_path = app
                        .state
                        .document
                        .active_doc_idx
                        .and_then(|idx| app.state.document.open_documents.get(idx))
                        .filter(|doc| !doc.is_reference)
                        .map(|doc| doc.path.to_path_buf());
                    let show_vertical_line = app
                        .state
                        .config
                        .settings
                        .settings()
                        .layout
                        .accordion_vertical_line;
                    let referenced_images = app
                        .state
                        .document
                        .active_doc_idx
                        .and_then(|idx| app.state.document.open_documents.get(idx))
                        .filter(|doc| !doc.is_reference && !doc.path.as_os_str().is_empty())
                        .map(|doc| {
                            let (_, paths) =
                                katana_core::preview::ImagePreviewOps::resolve_image_paths(
                                    &doc.buffer,
                                    &doc.path,
                                );
                            paths
                        })
                        .unwrap_or_default();
                    crate::views::panels::explorer::ExplorerPanel::new(
                        &mut app.state.workspace,
                        &mut app.state.search,
                        &app.state.global_workspace.state().histories,
                        active_path.as_deref(),
                        &app.state.document.tab_groups,
                        &mut app.pending_action,
                        show_vertical_line,
                    )
                    .with_referenced_images(referenced_images)
                    .show(ui);
                });

            /* WHY: Reset hover state while pinned so it starts clean after unpinning. */
            ui.ctx().data_mut(|d| d.insert_temp(hover_id, false));
            Self::render_rail_popup(ui, app);
            return;
        }

        /* Case 2: HOVER ONLY — float over content using Area::Foreground. */
        let cooldown_id = egui::Id::new("explorer_hover_cooldown");
        let explorer_hover_open = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(hover_id))
            .unwrap_or(false);

        let explorer_btn_rect = ui
            .ctx()
            .data(|d| d.get_temp::<egui::Rect>(egui::Id::new("explorer_btn_rect")));

        let in_cooldown: bool = ui.ctx().data(|d| d.get_temp(cooldown_id).unwrap_or(false));

        let over_btn = ui.input(|i| i.pointer.hover_pos()).is_some_and(|pos| {
            explorer_btn_rect
                .is_some_and(|r| r.expand(super::hover::EXPLORER_HOVER_MARGIN).contains(pos))
        });

        /* WHY: Re-arm hover once cursor leaves the button after a click (spec item 4). */
        if in_cooldown && !over_btn {
            ui.ctx().data_mut(|d| d.insert_temp(cooldown_id, false));
        }

        /* WHY: Open hover overlay when the pointer enters the explorer toggle button */
        if !explorer_hover_open && !in_cooldown && over_btn {
            ui.ctx().data_mut(|d| d.insert_temp(hover_id, true));
        }

        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("explorer_hover_anim"),
            explorer_hover_open,
            EXPLORER_ANIM_SPEED,
        );

        if anim > 0.0 {
            super::hover::ExplorerHoverOverlay::show(
                ui,
                app,
                anim,
                rail_resp.response.rect,
                explorer_btn_rect,
            );
        }

        Self::render_rail_popup(ui, app);
    }
}
