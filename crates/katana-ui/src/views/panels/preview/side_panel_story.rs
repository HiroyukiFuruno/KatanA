use super::side_panels::{
    PANEL_ANIM_SPEED, PANEL_HEAD_SPACE, PANEL_HOVER_MARGIN, PANEL_ITEM_SPACE, PANEL_WIDTH,
    POPUP_GAP, POPUP_PADDING, POPUP_ROUNDING, POPUP_SHADOW_ALPHA, PreviewSidePanels,
};
use crate::app_state::AppAction;
use eframe::egui;

const SLIDESHOW_BTN_H: f32 = 44.0;
const SLIDESHOW_BTN_ROUNDING: f32 = 8.0;
const STORY_RIGHT_MARGIN: f32 = 8.0;

impl<'a> PreviewSidePanels<'a> {
    /// Render the story/slideshow popup as a foreground overlay.
    pub(super) fn render_story(&mut self, ui: &mut egui::Ui) {
        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("story_panel_anim"),
            self.app.state.layout.show_story_panel,
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
        if self.app.state.layout.show_story_panel
            && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            && let Some(btn_rect) = self.story_btn_rect
            && btn_rect.expand(PANEL_HOVER_MARGIN).contains(pos)
        {
            keep_open = true;
        }

        /* WHY: Position the overlay to the left of the sidebar, floating on top. */
        let panel_x = sidebar_rect.left() - PANEL_WIDTH * anim - POPUP_GAP;
        let panel_y = sidebar_rect.top();
        let panel_height = sidebar_rect.height();

        let animation_f32 = anim;
        let area_resp = egui::Area::new(egui::Id::new("preview_story_overlay"))
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
                    self.render_story_inner(ui);
                });
            });

        if self.app.state.layout.show_story_panel {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let over_overlay = area_resp
                    .response
                    .rect
                    .expand(PANEL_HOVER_MARGIN)
                    .contains(pos);
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
                self.app.state.layout.show_story_panel = false;
            }
        }
    }

    fn render_story_inner(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(PANEL_HEAD_SPACE);

            ui.indent("story_content", |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = PANEL_ITEM_SPACE;
                    /* WHY: Reserve right margin so LabeledToggle doesn't clip. */
                    ui.set_max_width(ui.available_width() - STORY_RIGHT_MARGIN);
                    let i18n = crate::i18n::I18nOps::get();

                    let mut hover = self.app.state.layout.slideshow_hover_highlight;
                    let hover_resp = ui.add(
                        crate::widgets::LabeledToggle::new(
                            i18n.preview.highlight_hover.clone(),
                            &mut hover,
                        )
                        .position(crate::widgets::TogglePosition::Right)
                        .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                    );
                    if hover_resp.changed() {
                        self.app.pending_action = AppAction::ToggleSlideshowHoverHighlight;
                    }

                    let mut controls = self.app.state.layout.slideshow_show_diagram_controls;
                    let controls_resp = ui.add(
                        crate::widgets::LabeledToggle::new(
                            i18n.preview.show_diagram_controls.clone(),
                            &mut controls,
                        )
                        .position(crate::widgets::TogglePosition::Right)
                        .alignment(crate::widgets::ToggleAlignment::SpaceBetween),
                    );
                    if controls_resp.changed() {
                        self.app.pending_action = AppAction::ToggleSlideshowShowDiagramControls;
                    }

                    ui.add_space(PANEL_ITEM_SPACE);

                    let start_btn = egui::Button::image_and_text(
                        crate::Icon::Action.ui_image(ui, crate::icon::IconSize::Medium),
                        i18n.preview.toggle_slideshow.clone(),
                    )
                    .min_size(egui::vec2(ui.available_width(), SLIDESHOW_BTN_H))
                    .rounding(egui::Rounding::same(SLIDESHOW_BTN_ROUNDING as u8));

                    if ui.add(start_btn).clicked() {
                        self.app.pending_action = AppAction::ToggleSlideshow;
                    }
                });
            });
        });
    }
}
