/* WHY: Specialized logic for tab item rendering to maintain UI modularity and architectural clarity. */

use crate::views::top_bar::tab_bar::tab_item::TabItem;
use eframe::egui;

const TAB_MIN_TITLE_WIDTH: f32 = 48.0;
const DOCUMENT_TAB_PADDING_X: f32 = 8.0;
const DOCUMENT_TAB_ICON_GAP: f32 = 4.0;
const DOCUMENT_TAB_TITLE_CLOSE_GAP: f32 = 4.0;
const DOCUMENT_TAB_TITLE_RIGHT_PADDING: f32 = DOCUMENT_TAB_PADDING_X;
const DOCUMENT_TAB_CLOSE_RIGHT_MARGIN: f32 = 2.0;
const TAB_UNDERLINE_WIDTH: f32 = 2.0;
const TAB_UNDERLINE_OFFSET: f32 = 1.0;

impl<'a> TabItem<'a> {
    pub(crate) fn tab_width_from_parts(
        title_width: f32,
        close_width: f32,
        icon_width: f32,
        close_visible: bool,
    ) -> f32 {
        let close_area = if close_visible {
            DOCUMENT_TAB_TITLE_CLOSE_GAP + close_width + DOCUMENT_TAB_CLOSE_RIGHT_MARGIN
        } else {
            DOCUMENT_TAB_TITLE_RIGHT_PADDING
        };
        (DOCUMENT_TAB_PADDING_X + title_width + close_area + icon_width)
            .max(TAB_MIN_TITLE_WIDTH + close_area)
            .min(super::TAB_MAX_WIDTH)
    }

    pub(crate) fn parent_tab_width(
        &self,
        ui: &egui::Ui,
        title: &str,
        is_changelog: bool,
        close_visible: bool,
    ) -> f32 {
        let close_width = Self::close_width(ui);
        let icon_width = if is_changelog {
            crate::icon::IconSize::Small.to_vec2().x + DOCUMENT_TAB_ICON_GAP
        } else {
            0.0
        };
        let title_width = self.title_text_width(ui, title);
        Self::tab_width_from_parts(title_width, close_width, icon_width, close_visible)
    }

    pub(crate) fn parent_tab_height(&self, ui: &egui::Ui) -> f32 {
        ui.spacing()
            .interact_size
            .y
            .max(ui.text_style_height(&egui::TextStyle::Button))
    }

    pub(crate) fn close_width(ui: &egui::Ui) -> f32 {
        crate::icon::IconSize::Small.to_vec2().x + ui.spacing().button_padding.x * 2.0
    }

    pub(crate) fn render_title_button_at(
        &self,
        ui: &mut egui::Ui,
        title_rect: egui::Rect,
        title: &str,
        is_changelog: bool,
    ) -> egui::Response {
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(title_rect.left() + DOCUMENT_TAB_PADDING_X, title_rect.top()),
            title_rect.right_bottom(),
        );
        let mut label_left = content_rect.left();
        if is_changelog {
            let icon_size = crate::icon::IconSize::Small.to_vec2();
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(label_left + icon_size.x / 2.0, content_rect.center().y),
                icon_size,
            );
            ui.put(
                icon_rect,
                crate::Icon::Info.ui_image(ui, crate::icon::IconSize::Small),
            );
            label_left = icon_rect.right() + DOCUMENT_TAB_ICON_GAP;
        }
        let measured_text_width = self.title_text_width(ui, title);
        let available_text_width = (content_rect.right() - label_left).max(0.0);
        let text_width = measured_text_width.min(available_text_width);
        let label_rect = egui::Rect::from_center_size(
            egui::pos2(label_left + text_width / 2.0, content_rect.center().y),
            egui::vec2(text_width, content_rect.height()),
        );
        let text = egui::RichText::new(title)
            .color(self.title_color(ui))
            .text_style(egui::TextStyle::Button);
        let label = egui::Label::new(text).selectable(false);
        let label = if measured_text_width > available_text_width {
            label.truncate()
        } else {
            label
        };
        ui.put(label_rect, label)
    }

    pub(crate) fn render_close_button_at(
        &self,
        ui: &mut egui::Ui,
        close_rect: egui::Rect,
    ) -> Option<egui::Response> {
        let icon = if self.doc.is_pinned {
            crate::Icon::Pin
        } else {
            crate::Icon::Close
        };
        let icon_rect = egui::Rect::from_center_size(
            close_rect.center(),
            crate::icon::IconSize::Small.to_vec2(),
        );
        ui.put(icon_rect, icon.ui_image(ui, crate::icon::IconSize::Small));
        let response = ui
            .interact(
                close_rect,
                egui::Id::new("document_tab_close_button")
                    .with(self.idx)
                    .with(self.doc.path.to_string_lossy().to_string()),
                egui::Sense::click(),
            )
            .on_hover_text(crate::i18n::I18nOps::get().tab.close.clone());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                ui.is_enabled(),
                crate::i18n::I18nOps::get().tab.close.clone(),
            )
        });
        Some(response)
    }

    pub(crate) fn title_close_gap() -> f32 {
        DOCUMENT_TAB_TITLE_CLOSE_GAP
    }

    pub(crate) fn close_right_margin() -> f32 {
        DOCUMENT_TAB_CLOSE_RIGHT_MARGIN
    }

    fn title_text_width(&self, ui: &egui::Ui, title: &str) -> f32 {
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        ui.painter()
            .layout_no_wrap(title.to_owned(), font_id, ui.visuals().text_color())
            .size()
            .x
    }

    fn title_color(&self, ui: &egui::Ui) -> egui::Color32 {
        if self.is_active {
            ui.visuals().selection.stroke.color
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        }
    }

    pub(crate) fn draw_group_underline(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let Some(g) = self.group else { return };
        let base_color =
            egui::Color32::from_hex(&g.color_hex).unwrap_or(ui.visuals().widgets.active.bg_fill);
        let line_y = rect.bottom() - TAB_UNDERLINE_OFFSET;
        ui.painter().hline(
            rect.x_range(),
            line_y,
            egui::Stroke::new(TAB_UNDERLINE_WIDTH, base_color),
        );
    }
}
