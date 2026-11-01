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

#[cfg(test)]
#[path = "group_header_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "group_header_visible_tests.rs"]
mod visible_tests;

impl<'a> GroupHeader<'a> {
    #[allow(deprecated)]
    pub fn show(self, ui: &mut egui::Ui, tab_action: &mut Option<AppAction>) {
        let base_color = egui::Color32::from_hex(&self.g.color_hex)
            .unwrap_or(ui.visuals().widgets.active.bg_fill);
        let frame_fill = self.header_fill(base_color);
        let group_resp = self.render_frame(ui, base_color, frame_fill);

        if group_resp.clicked() {
            *tab_action = Some(AppAction::ToggleCollapseTabGroup(self.g.id.clone()));
        }
        if group_resp.secondary_clicked() && self.g.name != "demo" {
            egui::Popup::toggle_id(ui.ctx(), egui::Id::new("group_popup").with(&self.g.id));
        }

        self.handle_popup(ui, tab_action, &group_resp);
    }

    fn header_fill(&self, base: egui::Color32) -> egui::Color32 {
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
        let state = super::group_header_popup::GroupHeaderPopupState::new(&self.g.id);
        self.open_inline_rename_once(ui, state);
        self.show_popup(state, tab_action, group_resp);
        self.cancel_closed_inline_rename(ui, state, tab_action);
        self.clear_closed_popup_editor_state(ui, state);
    }

    fn open_inline_rename_once(
        &self,
        ui: &egui::Ui,
        state: super::group_header_popup::GroupHeaderPopupState,
    ) {
        if self.inline_rename_group == Some(&self.g.id) && self.g.name != "demo" {
            state.open_inline_once(ui.ctx());
        }
    }

    fn show_popup(
        &self,
        state: super::group_header_popup::GroupHeaderPopupState,
        tab_action: &mut Option<AppAction>,
        group_response: &egui::Response,
    ) {
        let rename = self.inline_rename_group;
        let _popup_response = egui::Popup::from_response(group_response)
            .id(state.popup_id)
            .open_memory(None)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                super::group_header_popup::GroupHeaderPopup {
                    g: self.g,
                    inline_rename_group: rename,
                    tab_action,
                    popup_id: state.popup_id,
                }
                .show(ui)
            });
    }

    fn cancel_closed_inline_rename(
        &self,
        ui: &egui::Ui,
        state: super::group_header_popup::GroupHeaderPopupState,
        tab_action: &mut Option<AppAction>,
    ) {
        let was_inline_rename = self.inline_rename_group == Some(&self.g.id);
        if was_inline_rename
            && state.inline_was_opened(ui.ctx())
            && !egui::Popup::is_id_open(ui.ctx(), state.popup_id)
            && tab_action.is_none()
        {
            *tab_action = Some(AppAction::FinishInlineRename {
                group_id: self.g.id.clone(),
                new_name: None,
            });
        }
    }

    fn clear_closed_popup_editor_state(
        &self,
        ui: &egui::Ui,
        state: super::group_header_popup::GroupHeaderPopupState,
    ) {
        if egui::Popup::is_id_open(ui.ctx(), state.popup_id) {
            return;
        }
        let is_inline_rename = self.inline_rename_group == Some(&self.g.id);
        state.clear_editor_state(ui.ctx());
        if !is_inline_rename {
            state.clear_open_marker(ui.ctx());
        }
    }
}
