pub mod centering {
    use egui::{remap, vec2, Rect, Response, Shape, Stroke, Ui};
    use std::f32::consts::TAU;

    /// Constant for optical Y-offset applied to the accordion triangle to counter native egui geometrical centering.
    const OPTICAL_Y_OFFSET: f32 = -4.0;

    /// Helper struct for custom UI drawing with optical corrections.
    pub struct AccordionIcon;

    impl AccordionIcon {
        /// Draws the default egui triangle icon, but applies a negative Y-offset to perfectly align the 
        /// geometric center of the icon to the optical crossbar cap height of typical Japanese body fonts.
        /// This resolves the issue where the native egui implementation causes the triangle to appear 
        /// completely bottom-aligned (touching the baseline) due to the large descender metrics in IMGUI parsing.
        pub fn paint_optically_centered(ui: &mut Ui, openness: f32, response: &Response) {
            let visuals = ui.style().interact(response);
            let mut rect = response.rect;

            // Optical offset: Shift the geometric box UP by OPTICAL_Y_OFFSET pixels.
            rect.set_center(egui::pos2(rect.center().x, rect.center().y + OPTICAL_Y_OFFSET));

            let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
            let rect = rect.expand(visuals.expansion);
            let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
            
            let rotation = egui::emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
            for p in &mut points {
                *p = rect.center() + rotation * (*p - rect.center());
            }

            ui.painter().add(Shape::convex_polygon(
                points,
                visuals.fg_stroke.color,
                Stroke::NONE,
            ));
        }
    }
}
