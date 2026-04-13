use crate::icon::{Icon, IconSize};
use crate::state::search::SearchParams;
use eframe::egui;

const SEARCH_PADDING_X: f32 = 8.0;
const SEARCH_PADDING_Y: f32 = 6.0;
const ROUNDING_RADIUS: f32 = 6.0;
const SEARCH_ITEM_SPACING: f32 = 4.0;
const ICON_OPACITY: f32 = 0.5;

pub enum SearchParamsRef<'a> {
    Full(&'a mut SearchParams),
    Simple(&'a mut String),
}

impl<'a> SearchParamsRef<'a> {
    pub fn query_mut(&mut self) -> &mut String {
        match self {
            Self::Full(p) => &mut p.query,
            Self::Simple(s) => s,
        }
    }
    pub fn query(&self) -> &str {
        match self {
            Self::Full(p) => &p.query,
            Self::Simple(s) => s,
        }
    }
    pub fn toggles(&mut self) -> Option<(&mut bool, &mut bool, &mut bool)> {
        match self {
            Self::Full(p) => Some((&mut p.match_case, &mut p.match_word, &mut p.use_regex)),
            Self::Simple(_) => None,
        }
    }
}

pub struct SearchBar<'a> {
    params: SearchParamsRef<'a>,
    hint_text: Option<egui::WidgetText>,
    desired_width: Option<f32>,
    text_color: Option<egui::Color32>,
    show_search_icon: bool,
    show_toggles: bool,
}

impl<'a> SearchBar<'a> {
    pub fn new(params: &'a mut SearchParams) -> Self {
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
            show_search_icon: false,
            show_toggles: false,
        }
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
                SEARCH_PADDING_X as i8,
                SEARCH_PADDING_Y as i8,
            ))
            .rounding(ROUNDING_RADIUS);

        let mut changed = false;
        /* WHY: Pre-compute row_height to constrain allocate_ui_with_layout call */
        let row_height = ui.spacing().interact_size.y;

        let inner_resp = frame.show(ui, |ui| {
            let mut text_response = None;
            /* WHY: Use available_width when no explicit width is requested */
            let content_width = self.desired_width.unwrap_or_else(|| ui.available_width());
            ui.set_max_height(row_height);
            ui.set_max_width(content_width);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = SEARCH_ITEM_SPACING;
                let has_toggles =
                    self.show_toggles && matches!(self.params, SearchParamsRef::Full(_));
                let query_empty = self.params.query().is_empty();
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
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if self.show_search_icon {
                        ui.add(
                            Icon::Search
                                .image(IconSize::Small)
                                .tint(ui.visuals().text_color().gamma_multiply(ICON_OPACITY)),
                        );
                    }
                    let mut text_edit = egui::TextEdit::singleline(self.params.query_mut())
                        .id_source("search_bar_text_edit")
                        .desired_width(f32::INFINITY)
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
                });
            });
            text_response.expect("TextEdit must be drawn")
        });

        let mut final_resp = inner_resp.inner;
        if final_resp.has_focus() {
            let stroke = egui::Stroke::new(1.0, ui.visuals().selection.bg_fill);
            /* WHY: Draw border inside frame bounds to prevent expansion */
            let rect = inner_resp.response.rect.shrink(stroke.width);
            ui.painter().add(egui::Shape::rect_stroke(
                rect,
                ROUNDING_RADIUS,
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
