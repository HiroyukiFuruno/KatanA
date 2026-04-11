use crate::app_state::AppAction;
use crate::state::document::TabGroup;
use eframe::egui;

const GROUP_HEADER_CORNER_RADIUS: u8 = 4;
const GROUP_HEADER_PADDING_X: i8 = 6;
const GROUP_HEADER_PADDING_Y: i8 = 3;
const GROUP_HEADER_COLLAPSED_ALPHA: u8 = 20;
const GROUP_HEADER_EXPANDED_ALPHA: u8 = 40;
const GROUP_HEADER_DOT_SIZE: f32 = 8.0;
const GROUP_HEADER_DOT_RADIUS: f32 = 4.0;
const GROUP_HEADER_ITEM_SPACING: f32 = 4.0;
const GROUP_HEADER_FONT_SIZE: f32 = 11.0;

pub(crate) struct GroupHeader<'a> {
    pub g: &'a TabGroup,
    pub inline_rename_group: Option<&'a String>,
}

impl<'a> GroupHeader<'a> {
    #[allow(deprecated)]
    pub fn show(self, ui: &mut egui::Ui, tab_action: &mut Option<AppAction>) {
        let base_color = egui::Color32::from_hex(&self.g.color_hex)
            .unwrap_or(ui.visuals().widgets.active.bg_fill);
        let frame_fill = self.header_fill(ui, base_color);
        let group_resp = self.render_frame(ui, base_color, frame_fill);

        if group_resp.clicked() {
            *tab_action = Some(AppAction::ToggleCollapseTabGroup(self.g.id.clone()));
        }
        if group_resp.secondary_clicked() && self.g.name != "demo" {
            ui.memory_mut(|mem| mem.toggle_popup(egui::Id::new("group_popup").with(&self.g.id)));
        }

        self.handle_popup(ui, tab_action, &group_resp);
    }

    fn header_fill(&self, _ui: &egui::Ui, base: egui::Color32) -> egui::Color32 {
        let alpha = if self.g.collapsed {
            GROUP_HEADER_COLLAPSED_ALPHA
        } else {
            GROUP_HEADER_EXPANDED_ALPHA
        };
        crate::theme_bridge::ThemeBridgeOps::from_rgba_unmultiplied(
            base.r(),
            base.g(),
            base.b(),
            alpha,
        )
    }

    #[allow(deprecated)]
    fn render_frame(
        &self,
        ui: &mut egui::Ui,
        base_color: egui::Color32,
        frame_fill: egui::Color32,
    ) -> egui::Response {
        egui::Frame::NONE
            .fill(frame_fill)
            .corner_radius(GROUP_HEADER_CORNER_RADIUS)
            .inner_margin(egui::Margin::symmetric(
                GROUP_HEADER_PADDING_X,
                GROUP_HEADER_PADDING_Y,
            ))
            .show(ui, |ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        ui.spacing_mut().item_spacing.x = GROUP_HEADER_ITEM_SPACING;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(GROUP_HEADER_DOT_SIZE, GROUP_HEADER_DOT_SIZE),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(
                            rect.center(),
                            GROUP_HEADER_DOT_RADIUS,
                            base_color,
                        );
                        ui.label(
                            egui::RichText::new(&self.g.name)
                                .color(ui.visuals().text_color())
                                .strong()
                                .size(GROUP_HEADER_FONT_SIZE),
                        );
                    })
                    .show(ui);
            })
            .response
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    }

    fn handle_popup(
        &self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
        group_resp: &egui::Response,
    ) {
        let popup_id = egui::Id::new("group_popup").with(&self.g.id);
        if self.inline_rename_group == Some(&self.g.id) && self.g.name != "demo" {
            ui.memory_mut(|mem| mem.open_popup(popup_id));
            *tab_action = Some(AppAction::ClearInlineRename);
        }
        let rename = self.inline_rename_group;
        let popup_resp = egui::popup_below_widget(
            ui,
            popup_id,
            group_resp,
            egui::PopupCloseBehavior::IgnoreClicks,
            |ui| {
                super::group_header_popup::GroupHeaderPopup {
                    g: self.g,
                    inline_rename_group: rename,
                    tab_action,
                    popup_id,
                }
                .show(ui)
            },
        );
        self.close_popup_on_outside_click(ui, popup_id, group_resp, popup_resp);
    }

    fn close_popup_on_outside_click(
        &self,
        ui: &mut egui::Ui,
        popup_id: egui::Id,
        group_resp: &egui::Response,
        popup_resp: Option<egui::Rect>,
    ) {
        let is_open = ui.memory(|mem| mem.is_popup_open(popup_id));
        let primary_pressed = ui.input(|i| i.pointer.any_pressed() && i.pointer.primary_pressed());
        if !is_open || !primary_pressed {
            return;
        }
        let Some(pos) = ui.input(|i| i.pointer.interact_pos()) else {
            return;
        };
        let outside_popup = popup_resp.is_none_or(|r| !r.contains(pos));
        let outside_header = !group_resp.rect.contains(pos);
        if outside_popup && outside_header {
            ui.memory_mut(|mem| mem.close_popup(popup_id));
        }
    }
}
