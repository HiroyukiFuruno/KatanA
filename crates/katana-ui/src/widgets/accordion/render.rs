use eframe::egui;

pub(crate) struct AccordionRenderer;

impl AccordionRenderer {
    pub(crate) fn render_background(
        active: bool,
        ui: &egui::Ui,
        rect: &egui::Rect,
        response: &egui::Response,
    ) {
        if active || response.hovered() {
            let bg_color = if active {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().widgets.hovered.bg_fill
            };

            let stroke = if response.hovered() && !active {
                ui.visuals().widgets.hovered.bg_stroke
            } else {
                egui::Stroke::NONE
            };

            ui.painter().rect(
                *rect,
                ui.style().visuals.widgets.hovered.corner_radius,
                bg_color,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
    }

    pub(crate) fn render_icon(
        active: bool,
        ui: &egui::Ui,
        rect: &egui::Rect,
        response: &egui::Response,
        state: &egui::collapsing_header::CollapsingState,
    ) -> egui::Rect {
        let icon_size = ui.spacing().icon_width;
        let icon_min_y = rect.center().y - icon_size / 2.0;
        let icon_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x, icon_min_y),
            egui::pos2(rect.min.x + icon_size, icon_min_y + icon_size),
        );

        let stroke_color = if active || response.hovered() || response.has_focus() {
            ui.style().visuals.widgets.hovered.fg_stroke.color
        } else {
            ui.style().interact(response).fg_stroke.color
        };

        const TRIANGLE_RADIUS_RATIO: f32 = 0.3;
        const TRIANGLE_BACK_RATIO: f32 = 0.6;
        let openness = state.openness(ui.ctx());

        let center = icon_rect.center();
        let triangle_radius = icon_size * TRIANGLE_RADIUS_RATIO;

        let rot = openness * std::f32::consts::FRAC_PI_2;
        let rot_mat = egui::emath::Rot2::from_angle(rot);
        let transform = |p: egui::Pos2| center + rot_mat * p.to_vec2();

        let points = vec![
            transform(egui::pos2(triangle_radius, 0.0)),
            transform(egui::pos2(
                -triangle_radius * TRIANGLE_BACK_RATIO,
                -triangle_radius,
            )),
            transform(egui::pos2(
                -triangle_radius * TRIANGLE_BACK_RATIO,
                triangle_radius,
            )),
        ];

        ui.painter().add(egui::Shape::convex_polygon(
            points,
            stroke_color,
            egui::Stroke::NONE,
        ));

        icon_rect
    }
}
