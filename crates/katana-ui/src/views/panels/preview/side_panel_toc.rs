use super::side_panels::{
    PANEL_ANIM_SPEED, PANEL_HOVER_MARGIN, PANEL_WIDTH, POPUP_GAP, POPUP_PADDING, POPUP_ROUNDING,
    POPUP_SHADOW_ALPHA, PreviewSidePanels,
};
use crate::shell_ui::{TOC_PANEL_DEFAULT_WIDTH, TOC_PANEL_MARGIN};
use eframe::egui;

#[inline]
fn toc_hover_id() -> egui::Id {
    egui::Id::new("toc_hover_open")
}

#[inline]
fn toc_cooldown_id() -> egui::Id {
    egui::Id::new("toc_hover_cooldown")
}

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_toc(&mut self, ui: &mut egui::Ui) {
        let toc_visible = self.app.state.config.settings.settings().layout.toc_visible;
        if !toc_visible {
            return;
        }

        let show_toc = self.app.state.layout.show_toc;

        /* WHY: Pinned mode — render TOC as a SidePanel that pushes content aside.
         * Hover state is not relevant here; cooldown is set at the click site in
         * side_panels.rs so that hover does not re-open immediately after unpinning. */
        if show_toc {
            let doc_path = self.app.state.active_document().map(|d| d.path.clone());
            self.render_toc_as_panel(ui, doc_path.as_deref());
            return;
        }

        /* show_toc=false: hover handling */
        let hover_pos = ui.input(|i| i.pointer.hover_pos());
        let over_btn = self
            .toc_btn_rect
            .zip(hover_pos)
            .is_some_and(|(r, pos)| r.expand(PANEL_HOVER_MARGIN).contains(pos));
        let ctx = ui.ctx().clone();
        let in_cooldown: bool = ctx.data(|d| d.get_temp(toc_cooldown_id()).unwrap_or(false));
        let current_open: bool = ctx.data(|d| d.get_temp(toc_hover_id()).unwrap_or(false));
        if in_cooldown && !over_btn {
            ctx.data_mut(|d| d.insert_temp(toc_cooldown_id(), false));
        }
        let should_open = over_btn && !in_cooldown && !current_open;
        if should_open {
            ctx.data_mut(|d| d.insert_temp(toc_hover_id(), true));
        }
        let toc_hover_open = should_open || current_open;
        if !toc_hover_open {
            return;
        }
        let anim = ctx.animate_bool_with_time(
            egui::Id::new("toc_panel_hover_anim"),
            true,
            PANEL_ANIM_SPEED,
        );
        if anim == 0.0 {
            return;
        }

        let sidebar_rect = match self.sidebar_rect {
            Some(r) => r,
            None => return,
        };
        let panel_x = sidebar_rect.left() - PANEL_WIDTH * anim - POPUP_GAP;
        let panel_y = sidebar_rect.top();
        let panel_height = sidebar_rect.height();
        let animation_f32 = anim;
        let mut clicked_line = None;
        let mut active_index_out = None;
        let doc_path = self.app.state.active_document().map(|d| d.path.clone());
        let area_resp = egui::Area::new(egui::Id::new("preview_toc_hover_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(panel_x, panel_y))
            .show(ui.ctx(), |ui| {
                let mut window_fill = ui.visuals().window_fill();
                window_fill = window_fill.gamma_multiply(animation_f32);
                let shadow_color = crate::theme_bridge::ThemeBridgeOps::from_black_alpha(
                    (animation_f32 * (POPUP_SHADOW_ALPHA as f32)) as u8,
                );
                let frame = egui::Frame::window(ui.style())
                    .fill(window_fill)
                    .shadow(egui::Shadow {
                        color: shadow_color,
                        ..Default::default()
                    })
                    .inner_margin(egui::Margin::same(POPUP_PADDING))
                    .rounding(POPUP_ROUNDING);
                frame.show(ui, |ui| {
                    ui.set_width(PANEL_WIDTH);
                    ui.set_min_height(panel_height);
                    if let Some(ref path) = doc_path
                        && let Some(preview) =
                            self.app.tab_previews.iter_mut().find(|p| &p.path == path)
                    {
                        let (cl, ai, _) = crate::views::panels::toc::TocPanel::new(
                            &mut preview.pane,
                            &mut self.app.state,
                        )
                        .show(ui);
                        clicked_line = cl;
                        active_index_out = ai;
                    }
                });
            });
        if let Some(clicked) = clicked_line {
            self.app.state.scroll.scroll_to_line = Some(clicked);
            self.app.state.scroll.last_scroll_to_line = None;
        }
        self.app.state.active_toc_index = active_index_out;
        let mut keep_open = false;
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            let over_overlay = area_resp
                .response
                .rect
                .expand(PANEL_HOVER_MARGIN)
                .contains(pos);
            let over_toc_btn = self
                .toc_btn_rect
                .is_some_and(|r| r.expand(PANEL_HOVER_MARGIN).contains(pos));
            if over_overlay || over_toc_btn {
                keep_open = true;
            }
        }
        if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
            ui.ctx().data_mut(|d| d.insert_temp(toc_hover_id(), false));
        }
    }

    fn render_toc_as_panel(&mut self, ui: &mut egui::Ui, doc_path: Option<&std::path::Path>) {
        use katana_platform::settings::TocPosition;
        let position = self
            .app
            .state
            .config
            .settings
            .settings()
            .layout
            .toc_position;
        let frame =
            egui::Frame::side_top_panel(&ui.ctx().global_style()).inner_margin(TOC_PANEL_MARGIN);
        let panel = match position {
            TocPosition::Left => egui::SidePanel::left("toc_panel"),
            TocPosition::Right => egui::SidePanel::right("toc_panel"),
        };
        let mut clicked_line = None;
        let mut active_index_out = None;
        let response = panel
            .frame(frame)
            .default_width(TOC_PANEL_DEFAULT_WIDTH)
            .show_inside(ui, |ui| {
                if let Some(path) = doc_path
                    && let Some(preview) = self.app.tab_previews.iter_mut().find(|p| p.path == path)
                {
                    let (cl, ai, _) = crate::views::panels::toc::TocPanel::new(
                        &mut preview.pane,
                        &mut self.app.state,
                    )
                    .show(ui);
                    clicked_line = cl;
                    active_index_out = ai;
                }
            });
        let edge = match position {
            TocPosition::Left => [
                response.response.rect.right_top(),
                response.response.rect.right_bottom(),
            ],
            TocPosition::Right => [
                response.response.rect.left_top(),
                response.response.rect.left_bottom(),
            ],
        };
        ui.painter().line_segment(edge, ui.visuals().window_stroke());
        if let Some(clicked) = clicked_line {
            self.app.state.scroll.scroll_to_line = Some(clicked);
            self.app.state.scroll.last_scroll_to_line = None;
        }
        self.app.state.active_toc_index = active_index_out;
    }
}
