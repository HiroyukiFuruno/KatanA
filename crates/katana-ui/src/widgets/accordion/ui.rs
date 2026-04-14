use super::types::*;
use crate::shell::{
    TREE_ACCORDION_LINE_DASH_LENGTH, TREE_ACCORDION_LINE_GAMMA, TREE_ACCORDION_LINE_GAP_LENGTH,
    TREE_ACCORDION_LINE_WIDTH,
};
use eframe::egui;

const ACCORDION_TEXT_SPACING: f32 = 4.0;

impl<'a> Accordion<'a> {
    pub fn show(self, ui: &mut egui::Ui) -> egui::InnerResponse<bool> {
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            self.id_source,
            self.default_open,
        );

        if let Some(force_open) = self.force_open {
            state.set_open(force_open);
        }

        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let icon_size = ui.spacing().icon_width;
        let spacing = ui.spacing().item_spacing.x;
        let text_max_width = ui.available_width() - icon_size - spacing;

        let galley = self.label.clone().into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            text_max_width.max(0.0),
            font_id,
        );

        let desired_size = egui::vec2(
            ui.available_width(),
            ui.spacing()
                .interact_size
                .y
                .max(icon_size)
                .max(galley.size().y),
        );

        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        let label_text = self.label.text();
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, label_text)
        });

        super::render::AccordionRenderer::render_background(self.active, ui, &rect, &response);
        let icon_rect = super::render::AccordionRenderer::render_icon(
            self.active,
            ui,
            &rect,
            &response,
            &state,
        );

        let icon_response =
            ui.interact(icon_rect, self.id_source.with("icon"), egui::Sense::click());

        let text_pos = egui::pos2(
            rect.min.x + icon_size + spacing,
            rect.center().y - galley.size().y / 2.0,
        );
        let final_text_color = if self.active {
            ui.visuals().selection.stroke.color
        } else if response.hovered() {
            ui.visuals().strong_text_color()
        } else if self.primary {
            ui.visuals().text_color()
        } else {
            ui.visuals().widgets.inactive.text_color()
        };
        ui.painter().galley(text_pos, galley, final_text_color);

        let clicked = if self.icon_only_toggle {
            icon_response.clicked()
        } else {
            response.clicked() || icon_response.clicked()
        };

        if clicked {
            state.toggle(ui);
        }

        /* WHY: Capture the vertical line start position */
        let line_start_y = rect.bottom();
        let mut line_end_y = line_start_y;

        let collapsing_resp = state.show_body_unindented(ui, |ui| {
            ui.add_space(ACCORDION_TEXT_SPACING);
            let indent = self.indent.unwrap_or_else(|| ui.spacing().indent);

            /* WHY: avoid ui.horizontal() to comply with AST lint, but use left_to_right layout
            directly to achieve the same indentation effect without the expanding positive feedback
            loop that AlignCenter causes. */
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.add_space(indent);
                ui.vertical(|ui| (self.body)(ui));
            });
        });

        if let Some(body_resp) = &collapsing_resp {
            line_end_y = body_resp.response.rect.bottom();
        }

        /* WHY: Render vertical hierarchy line if enabled and open. */
        if self.show_vertical_line && state.is_open() {
            let line_x = rect.left() + (icon_size / 2.0);
            let stroke = egui::Stroke::new(
                TREE_ACCORDION_LINE_WIDTH,
                ui.visuals()
                    .text_color()
                    .gamma_multiply(TREE_ACCORDION_LINE_GAMMA),
            );
            ui.painter().add(egui::Shape::dashed_line(
                &[
                    egui::pos2(line_x, line_start_y),
                    egui::pos2(line_x, line_end_y),
                ],
                stroke,
                TREE_ACCORDION_LINE_DASH_LENGTH,
                TREE_ACCORDION_LINE_GAP_LENGTH,
            ));
        }

        egui::InnerResponse::new(state.is_open(), response)
    }
}
