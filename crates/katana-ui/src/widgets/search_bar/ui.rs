use super::constants::*;
use super::types::*;
use crate::icon::{Icon, IconSize};
use crate::widgets::AlignCenter;
use eframe::egui;

pub struct SearchBar<'a> {
    pub(super) params: SearchParamsRef<'a>,
    pub(super) hint_text: Option<egui::WidgetText>,
    pub(super) desired_width: Option<f32>,
    pub(super) text_color: Option<egui::Color32>,
    pub(super) show_search_icon: bool,
    pub(super) show_toggles: bool,
}

impl<'a> SearchBar<'a> {
    pub fn new(params: &'a mut crate::state::search::SearchParams) -> Self {
        Self {
            params: SearchParamsRef::Full(params),
            hint_text: None,
            desired_width: None,
            text_color: None,
            show_search_icon: true,
            show_toggles: true,
        }
    }

    pub fn simple(query: &'a mut String) -> Self {
        Self {
            params: SearchParamsRef::Simple(query),
            hint_text: None,
            desired_width: None,
            text_color: None,
            show_search_icon: true,
            show_toggles: false,
        }
    }

    pub fn show_search_icon(mut self, show: bool) -> Self {
        self.show_search_icon = show;
        self
    }

    pub fn show_toggles(mut self, show: bool) -> Self {
        self.show_toggles = show;
        self
    }

    pub fn hint_text(mut self, hint: impl Into<egui::WidgetText>) -> Self {
        self.hint_text = Some(hint.into());
        self
    }

    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn text_color(mut self, color: egui::Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let changed = std::cell::Cell::new(false);

        let frame = egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(
                SEARCH_PADDING_X as i8,
                SEARCH_PADDING_Y as i8,
            ))
            .rounding(ROUNDING_RADIUS);

        let mut text_response = None;
        let params = std::cell::RefCell::new(self.params);

        let response = frame.show(ui, |ui| {
            let available_width = self.desired_width.unwrap_or_else(|| ui.available_width());
            let mut align = AlignCenter::new()
                .width(available_width)
                .spacing(SEARCH_ITEM_SPACING);

            if self.show_search_icon {
                align = align.left(|ui| {
                    ui.add(
                        Icon::Search
                            .image(IconSize::Small)
                            .tint(ui.visuals().text_color().gamma_multiply(ICON_OPACITY)),
                    )
                });
            }

            let has_toggles =
                self.show_toggles && matches!(*params.borrow(), SearchParamsRef::Full(_));
            let toggles_width = if has_toggles {
                (TOGGLE_BTN_WIDTH * TOGGLE_COUNT) + (SEARCH_ITEM_SPACING * (TOGGLE_COUNT - 1.0))
            } else {
                0.0
            };

            let query_empty = params.borrow().query().is_empty();
            let clear_btn_width = if !query_empty { CLEAR_BTN_WIDTH } else { 0.0 };
            let padding = if self.show_search_icon {
                ICON_LEFT_PADDING
            } else {
                ICON_NONE_PADDING
            };

            let text_edit_width =
                (available_width - toggles_width - clear_btn_width - padding).max(0.0);

            align = align.left(|ui| {
                let mut p = params.borrow_mut();
                let mut text_edit = egui::TextEdit::singleline(p.query_mut())
                    .desired_width(text_edit_width)
                    .frame(egui::Frame::none());
                if let Some(color) = self.text_color {
                    text_edit = text_edit.text_color(color);
                }
                if let Some(hint) = self.hint_text {
                    text_edit = text_edit.hint_text(hint);
                }
                let response = ui.add(text_edit);
                if response.changed() {
                    changed.set(true);
                }
                text_response = Some(response.clone());
                response
            });

            if !query_empty {
                align = align.right(|ui| {
                    let clear_resp = ui.add(
                        Icon::Close
                            .button(ui, IconSize::Small)
                            .min_size(egui::vec2(TOGGLE_BTN_WIDTH, TOGGLE_BTN_WIDTH)),
                    );
                    if clear_resp.clicked() {
                        params.borrow_mut().query_mut().clear();
                        changed.set(true);
                    }
                    clear_resp
                });
            }

            if has_toggles {
                align = align.right(|ui| {
                    let mut p = params.borrow_mut();
                    let Some((match_case, match_word, use_regex)) = p.toggles() else {
                        return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
                    };

                    let mut changed_local = false;
                    let mut toggle_btn = |ui: &mut egui::Ui, icon: Icon, is_active: &mut bool| {
                        let resp = ui.add(icon.selected_button(ui, IconSize::Small, *is_active));
                        if resp.clicked() {
                            *is_active = !*is_active;
                            changed_local = true;
                        }
                        resp
                    };

                    AlignCenter::new()
                        .spacing(SEARCH_ITEM_SPACING)
                        .shrink_to_fit(true)
                        .content(|ui| {
                            toggle_btn(ui, Icon::MatchCase, match_case);
                            toggle_btn(ui, Icon::WholeWord, match_word);
                            toggle_btn(ui, Icon::UseRegex, use_regex);
                        })
                        .show(ui);

                    if changed_local {
                        changed.set(true);
                    }
                    ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                });
            }

            align.show(ui)
        });

        let mut final_response = text_response.unwrap_or(response.response);
        if changed.get() {
            final_response.mark_changed();
        }
        final_response
    }
}
