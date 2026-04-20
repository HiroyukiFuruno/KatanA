/* WHY: Specialized logic for tab item rendering to maintain UI modularity and architectural clarity. */

use crate::views::top_bar::tab_bar::tab_item::TabItem;
use eframe::egui;

const TAB_UNDERLINE_WIDTH: f32 = 2.0;
const TAB_UNDERLINE_OFFSET: f32 = 1.0;

impl<'a> TabItem<'a> {
    pub(crate) fn render_title_button(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        is_changelog: bool,
    ) -> egui::Response {
        if is_changelog {
            ui.add(
                egui::Button::image_and_text(
                    crate::Icon::Info.ui_image(ui, crate::icon::IconSize::Small),
                    title,
                )
                .selected(self.is_active)
                .frame(false),
            )
        } else {
            ui.add(egui::Button::selectable(self.is_active, title).frame(false))
        }
    }

    pub(crate) fn render_close_button(&self, ui: &mut egui::Ui) -> Option<egui::Response> {
        let btn = if self.doc.is_pinned {
            crate::Icon::Pin.button(ui, crate::icon::IconSize::Small)
        } else {
            crate::Icon::Close.button(ui, crate::icon::IconSize::Small)
        };
        Some(ui.add(btn))
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
