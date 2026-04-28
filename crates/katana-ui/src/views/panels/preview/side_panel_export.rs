use super::side_panels::{
    PANEL_ANIM_SPEED, PANEL_HEAD_SPACE, PANEL_HOVER_MARGIN, PANEL_WIDTH, POPUP_GAP, POPUP_PADDING,
    POPUP_ROUNDING, POPUP_SHADOW_ALPHA, PreviewSidePanels,
};
use crate::app_state::AppAction;
use eframe::egui;

const EXPORT_BTN_H: f32 = 32.0;
const EXPORT_ITEM_SPACE: f32 = 4.0;

impl<'a> PreviewSidePanels<'a> {
    /// Render the export popup as a foreground overlay (no layout compression).
    pub(super) fn render_export(&mut self, ui: &mut egui::Ui) {
        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("export_panel_anim"),
            self.app.state.layout.show_export_panel,
            PANEL_ANIM_SPEED,
        );

        if anim == 0.0 {
            return;
        }

        let sidebar_rect = match self.sidebar_rect {
            Some(r) => r,
            None => return,
        };

        let mut keep_open = false;
        /* WHY: Keep panel open while pointer is near the toggle button. */
        if self.app.state.layout.show_export_panel
            && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            && let Some(btn_rect) = self.export_btn_rect
            && btn_rect.expand(PANEL_HOVER_MARGIN).contains(pos)
        {
            keep_open = true;
        }

        /* WHY: Position the overlay to the left of the sidebar, floating on top. */
        let panel_x = sidebar_rect.left() - PANEL_WIDTH * anim - POPUP_GAP;
        let panel_y = sidebar_rect.top();
        let panel_height = sidebar_rect.height();
        let panel_rect = egui::Rect::from_min_size(
            egui::pos2(panel_x, panel_y),
            egui::vec2(PANEL_WIDTH, panel_height),
        );
        crate::widgets::InteractionFacade::consume_rect(
            ui,
            "preview_export_overlay_input_blocker",
            panel_rect,
        );

        let animation_f32 = anim;
        let area_resp = egui::Area::new(egui::Id::new("preview_export_overlay"))
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
                    self.render_export_inner(ui);
                });
            });

        if self.app.state.layout.show_export_panel {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let over_overlay = area_resp
                    .response
                    .rect
                    .expand(PANEL_HOVER_MARGIN)
                    .contains(pos);
                /* WHY: Don't close while mouse is on ANY popup toggle button;
                the hover-delay logic in handle_popup_hover needs the current
                panel to stay alive during the switch delay. */
                let over_any_btn = [
                    self.export_btn_rect,
                    self.story_btn_rect,
                    self.tools_btn_rect,
                ]
                .iter()
                .any(|r| r.is_some_and(|rect| rect.expand(PANEL_HOVER_MARGIN).contains(pos)));

                if over_overlay || over_any_btn {
                    keep_open = true;
                }
            }

            if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
                self.app.state.layout.show_export_panel = false;
            }
        }
    }

    fn render_export_inner(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(PANEL_HEAD_SPACE);

            let i18n = crate::i18n::I18nOps::get();
            let formats = [
                (
                    crate::Icon::Html,
                    i18n.menu.export_html.clone(),
                    crate::app_state::ExportFormat::Html,
                ),
                (
                    crate::Icon::Pdf,
                    i18n.menu.export_pdf.clone(),
                    crate::app_state::ExportFormat::Pdf,
                ),
                (
                    crate::Icon::Image,
                    i18n.menu.export_png.clone(),
                    crate::app_state::ExportFormat::Png,
                ),
                (
                    crate::Icon::Image,
                    i18n.menu.export_jpg.clone(),
                    crate::app_state::ExportFormat::Jpg,
                ),
            ];

            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = EXPORT_ITEM_SPACE;
                for (icon, label, fmt) in formats {
                    let resp = ui.add(
                        egui::Button::image_and_text(
                            icon.ui_image(ui, crate::icon::IconSize::Medium),
                            label,
                        )
                        .fill(crate::theme_bridge::TRANSPARENT)
                        .min_size(egui::vec2(ui.available_width(), EXPORT_BTN_H)),
                    );
                    if resp.clicked() {
                        self.app.pending_action = AppAction::ExportDocument(fmt);
                    }
                }
            });
        });
    }
}
