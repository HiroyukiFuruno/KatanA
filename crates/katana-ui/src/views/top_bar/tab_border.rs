use eframe::egui;

const TAB_BORDER_WIDTH: f32 = 1.0;
const TAB_BORDER_FALLBACK_OPACITY: f32 = 0.32;
const TAB_BORDER_MIN_VISIBLE_ALPHA: u8 = 30;

pub(crate) struct TabBorderOps;

impl TabBorderOps {
    pub(crate) fn paint(ui: &egui::Ui, rect: egui::Rect, hovered: bool) {
        Self::paint_with_radius(ui, rect, hovered, egui::CornerRadius::ZERO);
    }

    pub(crate) fn paint_with_radius(
        ui: &egui::Ui,
        rect: egui::Rect,
        hovered: bool,
        corner_radius: egui::CornerRadius,
    ) {
        ui.painter().add(egui::Shape::rect_stroke(
            rect,
            corner_radius,
            Self::stroke(ui, hovered),
            Self::stroke_kind(),
        ));
    }

    fn stroke(ui: &egui::Ui, hovered: bool) -> egui::Stroke {
        egui::Stroke::new(TAB_BORDER_WIDTH, Self::color(ui, hovered))
    }

    fn color(ui: &egui::Ui, hovered: bool) -> egui::Color32 {
        let fallback_color = ui
            .visuals()
            .text_color()
            .linear_multiply(TAB_BORDER_FALLBACK_OPACITY);
        Self::resolve_color(
            hovered,
            ui.visuals().selection.bg_fill,
            ui.visuals().window_stroke(),
            fallback_color,
        )
    }

    pub(crate) fn resolve_color(
        hovered: bool,
        accent_color: egui::Color32,
        base_stroke: egui::Stroke,
        fallback_color: egui::Color32,
    ) -> egui::Color32 {
        if hovered {
            return accent_color;
        }
        if Self::is_visible_stroke(base_stroke) {
            return base_stroke.color;
        }
        fallback_color
    }

    pub(crate) fn is_visible_stroke(stroke: egui::Stroke) -> bool {
        stroke.width > 0.0 && stroke.color.a() >= TAB_BORDER_MIN_VISIBLE_ALPHA
    }

    pub(crate) fn stroke_kind() -> egui::StrokeKind {
        egui::StrokeKind::Inside
    }

    pub(crate) fn rect_contains_pointer(ui: &egui::Ui, rect: egui::Rect) -> bool {
        ui.input(|input| {
            input
                .pointer
                .hover_pos()
                .is_some_and(|pos| rect.contains(pos))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TabBorderOps;
    use crate::theme_bridge::ThemeBridgeOps;
    use eframe::egui;

    #[test]
    fn border_is_drawn_inside_tab_bounds() {
        assert!(matches!(
            TabBorderOps::stroke_kind(),
            egui::StrokeKind::Inside
        ));
    }

    #[test]
    fn hovered_border_uses_accent_color() {
        let accent = ThemeBridgeOps::from_rgb(20, 120, 220);
        let base = egui::Stroke::new(1.0, ThemeBridgeOps::from_rgb(80, 80, 80));
        let fallback = ThemeBridgeOps::from_rgb(160, 160, 160);

        assert_eq!(
            TabBorderOps::resolve_color(true, accent, base, fallback),
            accent
        );
    }

    #[test]
    fn transparent_tab_border_stroke_is_not_visible() {
        let stroke = egui::Stroke::new(1.0, ThemeBridgeOps::from_rgba_unmultiplied(80, 80, 80, 1));

        assert!(!TabBorderOps::is_visible_stroke(stroke));
    }

    #[test]
    fn opaque_tab_border_stroke_is_visible() {
        let stroke =
            egui::Stroke::new(1.0, ThemeBridgeOps::from_rgba_unmultiplied(80, 80, 80, 180));

        assert!(TabBorderOps::is_visible_stroke(stroke));
    }

    #[test]
    fn transparent_base_stroke_uses_fallback_color() {
        let accent = ThemeBridgeOps::from_rgb(20, 120, 220);
        let base = egui::Stroke::new(1.0, ThemeBridgeOps::from_rgba_unmultiplied(80, 80, 80, 1));
        let fallback = ThemeBridgeOps::from_rgb(160, 160, 160);

        assert_eq!(
            TabBorderOps::resolve_color(false, accent, base, fallback),
            fallback
        );
    }
}
