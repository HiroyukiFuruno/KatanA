use super::side_panels::{
    PANEL_ANIM_SPEED, PANEL_HOVER_MARGIN, PANEL_WIDTH, POPUP_GAP, POPUP_PADDING, POPUP_ROUNDING,
    POPUP_SHADOW_ALPHA, PreviewSidePanels,
};
use eframe::egui;

impl<'a> PreviewSidePanels<'a> {
    /// Render the tools popup as a foreground overlay.
    pub(super) fn render_tools(&mut self, ui: &mut egui::Ui) {
        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("tools_panel_anim"),
            self.app.state.layout.show_tools_panel,
            PANEL_ANIM_SPEED,
        );

        if anim == 0.0 {
            return;
        }

        let sidebar_rect = match self.sidebar_rect {
            Some(r) => r,
            None => return,
        };

        /* WHY: Position the overlay to the left of the sidebar, floating on top. */
        let panel_x = sidebar_rect.left() - PANEL_WIDTH * anim - POPUP_GAP;
        let panel_y = sidebar_rect.top();
        let panel_height = sidebar_rect.height();

        let animation_f32 = anim;
        let area_resp = egui::Area::new(egui::Id::new("preview_tools_overlay"))
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
                    self.render_tools_inner(ui);
                });
            });

        if self.app.state.layout.show_tools_panel
            && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
        {
            let over_any_btn = [
                self.export_btn_rect,
                self.story_btn_rect,
                self.tools_btn_rect,
            ]
            .iter()
            .any(|r| r.is_some_and(|rect| rect.expand(PANEL_HOVER_MARGIN).contains(pos)));
            let panel_hover = area_resp
                .response
                .rect
                .expand(PANEL_HOVER_MARGIN)
                .contains(pos);
            if !over_any_btn && !panel_hover {
                self.app.state.layout.show_tools_panel = false;
            }
        }
    }
}
