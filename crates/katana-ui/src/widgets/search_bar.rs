use crate::icon::{Icon, IconSize};
use eframe::egui;

mod layout;
mod params;

use params::SearchParamsRef;

const ICON_OPACITY: f32 = 0.5;

pub struct SearchBar<'a> {
    params: SearchParamsRef<'a>,
    hint_text: Option<egui::WidgetText>,
    desired_width: Option<f32>,
    text_color: Option<egui::Color32>,
    show_search_icon: bool,
    show_toggles: bool,
    id_source: Option<egui::Id>,
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
            id_source: None,
        }
    }

    pub fn simple(query: &'a mut String) -> Self {
        Self {
            params: SearchParamsRef::Simple(query),
            hint_text: None,
            desired_width: None,
            text_color: None,
            show_search_icon: false,
            show_toggles: false,
            id_source: None,
        }
    }

    pub fn id_source(mut self, id: impl std::hash::Hash) -> Self {
        self.id_source = Some(egui::Id::new(id));
        self
    }

    pub fn show_search_icon(mut self, enabled: bool) -> Self {
        self.show_search_icon = enabled;
        self
    }

    pub fn show_toggles(mut self, enabled: bool) -> Self {
        self.show_toggles = enabled;
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

    pub fn show(mut self, ui: &mut egui::Ui) -> egui::Response {
        let frame = egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(
                layout::PADDING_X as i8,
                layout::PADDING_Y as i8,
            ))
            .rounding(layout::ROUNDING_RADIUS);

        let mut changed = false;
        /* WHY: Pre-compute row_height to constrain allocate_ui_with_layout call */
        let row_height = ui.spacing().interact_size.y;

        let inner_resp = frame.show(ui, |ui| {
            let mut text_response = None;
            let content_width = layout::content_width(ui, self.desired_width);
            ui.set_max_height(row_height);
            ui.set_min_width(content_width);
            ui.set_max_width(content_width);

            /* WHY: Determine ID FIRST, before drawing any conditional widgets that could offset next_auto_id */
            let id_source = self.id_source.unwrap_or_else(|| ui.next_auto_id());
            let has_toggles = self.show_toggles && matches!(self.params, SearchParamsRef::Full(_));
            let query_empty = self.params.query().is_empty();

            crate::widgets::AlignCenter::new()
                .spacing(layout::ITEM_SPACING)
                .width(content_width)
                .left(|ui| {
                let icon_slot_width = ui.spacing().icon_width;
                if self.show_search_icon {
                    ui.add_sized(
                        [icon_slot_width, row_height],
                        Icon::Search
                            .image(IconSize::Small)
                            .tint(ui.visuals().text_color().gamma_multiply(ICON_OPACITY)),
                    );
                } else {
                    /* WHY: Preserve text alignment when the icon is hidden. */
                    ui.add_space(icon_slot_width);
                }

                let trailing_width = layout::trailing_width(ui, has_toggles, !query_empty);
                let text_width =
                    (ui.available_width() - trailing_width).max(layout::MIN_TEXT_INPUT_WIDTH);
                let mut text_edit = egui::TextEdit::singleline(self.params.query_mut())
                    .id_source(id_source)
                    .desired_width(text_width)
                    .frame(egui::Frame::none());
                if let Some(color) = self.text_color {
                    text_edit = text_edit.text_color(color);
                }
                if let Some(hint) = self.hint_text.take() {
                    text_edit = text_edit.hint_text(hint);
                }
                let resp = ui.add(text_edit);
                if resp.changed() {
                    changed = true;
                }
                text_response = Some(resp);

                if !query_empty && ui.add(Icon::Close.button(ui, IconSize::Small)).clicked() {
                    self.params.query_mut().clear();
                    changed = true;
                }
                if has_toggles
                    && let Some((match_case, match_word, use_regex)) = self.params.toggles()
                {
                    let mut toggle_btn = |ui: &mut egui::Ui, icon: Icon, is_active: &mut bool| {
                        if ui
                            .add(icon.selected_button(ui, IconSize::Small, *is_active))
                            .clicked()
                        {
                            *is_active = !*is_active;
                            changed = true;
                        }
                    };
                    toggle_btn(ui, Icon::UseRegex, use_regex);
                    toggle_btn(ui, Icon::WholeWord, match_word);
                    toggle_btn(ui, Icon::MatchCase, match_case);
                }
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);
            text_response.expect("TextEdit must be drawn")
        });

        let mut final_resp = inner_resp.inner;
        if final_resp.has_focus() {
            let stroke = egui::Stroke::new(1.0, ui.visuals().selection.bg_fill);
            /* WHY: Draw border inside frame bounds to prevent expansion */
            let rect = inner_resp.response.rect.shrink(stroke.width);
            ui.painter().add(egui::Shape::rect_stroke(
                rect,
                layout::ROUNDING_RADIUS,
                stroke,
                egui::StrokeKind::Inside,
            ));
        }
        if changed {
            final_resp.mark_changed();
        }
        final_resp
    }
}
