use super::types::*;
use crate::shell_ui::TOC_INDENT_PER_LEVEL;
use eframe::egui;

const TOC_ROW_HEIGHT: f32 = 24.0;
const TOC_FONT_SIZE: f32 = 13.0;

impl<'a> TocPanel<'a> {
    pub(crate) fn render_toc_item(
        &self,
        ui: &mut egui::Ui,
        ctx: &TocRenderContext<'_>,
        idx: &mut usize,
    ) -> Option<usize> {
        let item = &ctx.items[*idx];
        let is_active = item.index == ctx.active_index;
        let mut next_scroll = None;

        let start_idx = *idx;
        let mut end_idx = start_idx + 1;
        while end_idx < ctx.items.len() && ctx.items[end_idx].level > item.level {
            end_idx += 1;
        }
        let has_children = end_idx > start_idx + 1;

        if !has_children {
            next_scroll = self.render_leaf_item(ui, ctx, item, is_active, idx, next_scroll);
        } else {
            next_scroll =
                self.render_parent_item(ui, ctx, item, is_active, end_idx, idx, next_scroll);
        }

        next_scroll
    }

    fn render_leaf_item(
        &self,
        ui: &mut egui::Ui,
        _ctx: &TocRenderContext<'_>,
        item: &katana_core::markdown::outline::OutlineItem,
        is_active: bool,
        idx: &mut usize,
        mut next_scroll: Option<usize>,
    ) -> Option<usize> {
        let id = ui.make_persistent_id(("toc_item", item.index));
        let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), TOC_ROW_HEIGHT));
        let response = ui
            .interact(rect, id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &item.text));

        if response.clicked() {
            next_scroll = Some(item.index);
        }

        if !ui.is_rect_visible(rect) {
            *idx += 1;
            return next_scroll;
        }

        self.render_row_background(ui, rect, is_active, &response);

        let mut text = egui::RichText::new(&item.text).size(TOC_FONT_SIZE);
        if is_active || item.level == 1 {
            text = text.strong();
        }

        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let text_offset = ui.spacing().icon_width + ui.spacing().item_spacing.x;
        let galley = egui::WidgetText::from(text).into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            rect.width() - text_offset,
            font_id,
        );

        let text_pos = egui::pos2(
            rect.min.x + text_offset,
            rect.center().y - galley.size().y / 2.0,
        );

        let text_color = if is_active {
            ui.visuals().selection.stroke.color
        } else if response.hovered() {
            ui.visuals().strong_text_color()
        } else if item.level <= 2 {
            ui.visuals().text_color()
        } else {
            ui.visuals().widgets.inactive.text_color()
        };

        ui.painter().galley(text_pos, galley, text_color);
        *idx += 1;

        next_scroll
    }

    #[allow(clippy::too_many_arguments)]
    fn render_parent_item(
        &self,
        ui: &mut egui::Ui,
        ctx: &TocRenderContext<'_>,
        item: &katana_core::markdown::outline::OutlineItem,
        is_active: bool,
        end_idx: usize,
        idx: &mut usize,
        mut next_scroll: Option<usize>,
    ) -> Option<usize> {
        let id = ui.make_persistent_id(("toc", item.index));

        let mut text = egui::RichText::new(&item.text).size(TOC_FONT_SIZE);
        if is_active || item.level == 1 {
            text = text.strong();
        }

        let mut inner_scroll = None;
        let accordion_response = crate::widgets::Accordion::new(id, text, |ui| {
            *idx += 1;
            while *idx < end_idx {
                if let Some(scroll) = self.render_toc_item(ui, ctx, idx) {
                    inner_scroll = Some(scroll);
                }
            }
        })
        .primary(item.level <= 2)
        .icon_only_toggle(true)
        .default_open(true)
        .active(is_active)
        .force_open(ctx.force_open)
        .show_vertical_line(ctx.show_vertical_lines)
        .indent(TOC_INDENT_PER_LEVEL)
        .show(ui);

        /* WHY: Accordion header click handling. Extract click from header to enable jumping. */
        if accordion_response.response.clicked() {
            next_scroll = Some(item.index);
        }

        /* WHY: Assigning *idx = end_idx unconditionally prevents infinite loops when
        animation out evaluates is_open() to false but still executes body up to end_idx. */
        *idx = end_idx;

        if let Some(scroll) = inner_scroll {
            next_scroll = Some(scroll);
        }

        next_scroll
    }

    fn render_row_background(
        &self,
        ui: &egui::Ui,
        rect: egui::Rect,
        is_active: bool,
        response: &egui::Response,
    ) {
        if is_active || response.hovered() {
            let bg_color = if is_active {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.hovered.bg_fill
            };

            let stroke = if response.hovered() && !is_active {
                ui.visuals().widgets.hovered.bg_stroke
            } else {
                egui::Stroke::NONE
            };

            ui.painter().rect(
                rect,
                ui.style().visuals.widgets.hovered.corner_radius,
                bg_color,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
    }
}
