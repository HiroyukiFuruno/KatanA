use super::side_panels::{
    PreviewSidePanels, PANEL_ANIM_SPEED, PANEL_HEAD_SPACE, PANEL_HOVER_MARGIN, PANEL_ITEM_SPACE,
    PANEL_WIDTH,
};
use crate::app_state::AppAction;
use eframe::egui;

const SLIDESHOW_BTN_H: f32 = 44.0;
const SLIDESHOW_BTN_ROUNDING: f32 = 8.0;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_story(&mut self, ui: &mut egui::Ui) {
        let anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("story_panel_anim"),
            self.app.state.layout.show_story_panel,
            PANEL_ANIM_SPEED,
        );

        if anim == 0.0 {
            return;
        }

        let mut keep_open = false;
        /* WHY: Keep panel open while pointer is near the toggle button. */
        if self.app.state.layout.show_story_panel
            && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
            && let Some(btn_rect) = self.story_btn_rect
            && btn_rect.expand(PANEL_HOVER_MARGIN).contains(pos)
        {
            keep_open = true;
        }

        let panel_resp = egui::SidePanel::right("preview_story_panel")
            .resizable(false)
            .exact_width(PANEL_WIDTH * anim)
            .show_inside(ui, |ui| {
                ui.set_min_width(PANEL_WIDTH);
                ui.vertical(|ui| {
                    ui.add_space(PANEL_HEAD_SPACE);

                    ui.indent("story_content", |ui| {
                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing.y = PANEL_ITEM_SPACE;
                            /* WHY: Reserve right margin so LabeledToggle doesn't clip. */
                            let right_margin = 8.0;
                            ui.set_max_width(ui.available_width() - right_margin);
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
                                self.app.pending_action =
                                    AppAction::ToggleSlideshowShowDiagramControls;
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
            });

        if self.app.state.layout.show_story_panel {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                && panel_resp.response.rect.expand(PANEL_HOVER_MARGIN).contains(pos)
            {
                keep_open = true;
            }

            if !keep_open && ui.input(|i| i.pointer.hover_pos().is_some()) {
                self.app.state.layout.show_story_panel = false;
            }
        }
    }
}
