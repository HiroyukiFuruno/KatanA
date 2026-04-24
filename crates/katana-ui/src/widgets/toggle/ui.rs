use super::types::*;

pub const TOGGLE_WIDTH_RATIO: f32 = 2.0;
pub const TOGGLE_RADIUS_RATIO: f32 = 0.5;
pub const TOGGLE_CIRCLE_RATIO: f32 = 0.75;
pub const TOGGLE_ATTACHED_DEFAULT_MARGIN: f32 = 8.0;

impl ToggleOps {
    pub fn switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
        let desired_size = ui.spacing().interact_size.y * egui::vec2(TOGGLE_WIDTH_RATIO, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
        });

        Self::paint_switch(ui, rect, &response, *on);

        response
    }

    pub fn paint_switch(ui: &mut egui::Ui, rect: egui::Rect, response: &egui::Response, on: bool) {
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool(response.id, on);
            let visuals = ui.style().interact_selectable(response, on);
            let rect = rect.expand(visuals.expansion);
            let radius = TOGGLE_RADIUS_RATIO * rect.height();

            let mut track_stroke = visuals.bg_stroke;
            const DIM_ALPHA_THRESHOLD: u8 = 30;
            if track_stroke.is_empty() || track_stroke.color.a() < DIM_ALPHA_THRESHOLD {
                let hovered_stroke = ui.style().visuals.widgets.hovered.bg_stroke;
                /* WHY: The user requested the border to be as visible as it is on hover to improve discoverability */
                track_stroke = egui::Stroke::new(1.0, hovered_stroke.color);
            }

            ui.painter().rect(
                rect,
                radius,
                visuals.bg_fill,
                track_stroke,
                egui::StrokeKind::Inside,
            );
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui.painter().circle(
                center,
                TOGGLE_CIRCLE_RATIO * radius,
                visuals.bg_fill,
                visuals.fg_stroke,
            );
        }
    }
}

impl<'a> egui::Widget for LabeledToggle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let text_galley = self.text.clone().into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            ui.available_width(),
            egui::TextStyle::Body,
        );
        let text_size = text_galley.size();
        let toggle_size = ui.spacing().interact_size.y * egui::vec2(TOGGLE_WIDTH_RATIO, 1.0);

        let margin = match self.alignment {
            ToggleAlignment::Attached(m) => m,
            ToggleAlignment::SpaceBetween => 0.0,
        };

        let desired_width = match self.alignment {
            ToggleAlignment::Attached(_) => {
                let h_pad = ui.spacing().button_padding.x * 2.0;
                text_size.x + toggle_size.x + margin + h_pad
            }
            ToggleAlignment::SpaceBetween => ui.available_width(),
        };

        let row_pad = ui.spacing().button_padding.y;
        let row_height = text_size
            .y
            .max(toggle_size.y)
            .max(ui.spacing().interact_size.y)
            + row_pad * 2.0;

        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(desired_width, row_height), egui::Sense::click());

        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }

        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *self.on, "")
        });

        if response.hovered() {
            ui.painter().rect_filled(
                rect,
                ui.style().visuals.widgets.hovered.corner_radius,
                ui.style().visuals.widgets.hovered.bg_fill,
            );
        }

        let (text_pos, toggle_pos) = match (self.position, self.alignment) {
            (TogglePosition::Right, ToggleAlignment::SpaceBetween) => {
                let text_x = rect.left();
                let toggle_x = rect.right() - toggle_size.x;
                (text_x, toggle_x)
            }
            (TogglePosition::Left, ToggleAlignment::SpaceBetween) => {
                let toggle_x = rect.left();
                let text_x = rect.right() - text_size.x;
                (text_x, toggle_x)
            }
            (TogglePosition::Right, ToggleAlignment::Attached(_)) => {
                let h_pad = ui.spacing().button_padding.x;
                let text_x = rect.left() + h_pad;
                let toggle_x = text_x + text_size.x + margin;
                (text_x, toggle_x)
            }
            (TogglePosition::Left, ToggleAlignment::Attached(_)) => {
                let h_pad = ui.spacing().button_padding.x;
                let toggle_x = rect.left() + h_pad;
                let text_x = toggle_x + toggle_size.x + margin;
                (text_x, toggle_x)
            }
        };

        let text_pos = egui::pos2(text_pos, rect.center().y - text_size.y / 2.0);
        let toggle_pos = egui::pos2(toggle_pos, rect.center().y - toggle_size.y / 2.0);

        let toggle_rect = egui::Rect::from_min_size(toggle_pos, toggle_size);

        let text_color = ui.style().interact(&response).text_color();
        ui.painter().galley(text_pos, text_galley, text_color);

        ToggleOps::paint_switch(ui, toggle_rect, &response, *self.on);

        response
    }
}
