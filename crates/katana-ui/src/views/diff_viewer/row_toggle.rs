use super::row::{CELL_MARGIN_X, CELL_MARGIN_Y, LINE_NUMBER_WIDTH, ROW_HEIGHT};
use super::style::DiffViewerPalette;
use eframe::egui;

const COLLAPSE_ICON_WIDTH: f32 = 12.0;
const COLLAPSE_ICON_HEIGHT: f32 = 8.0;
const COLLAPSE_ICON_OFFSET_Y: f32 = 4.0;
const COLLAPSE_ICON_LEFT_PADDING: f32 = 4.0;

pub(super) struct DiffViewerRowToggleOps;

impl DiffViewerRowToggleOps {
    pub(super) fn show(ui: &mut egui::Ui, palette: &DiffViewerPalette, expanded: bool) -> bool {
        Self::show_with_label(ui, palette, palette.collapsed_background, expanded, None)
    }

    pub(super) fn show_gutter(
        ui: &mut egui::Ui,
        palette: &DiffViewerPalette,
        expanded: bool,
        label: &str,
    ) -> bool {
        Self::show_with_label(
            ui,
            palette,
            palette.gutter_background,
            expanded,
            Some(label),
        )
    }

    fn show_with_label(
        ui: &mut egui::Ui,
        palette: &DiffViewerPalette,
        fill: egui::Color32,
        expanded: bool,
        label: Option<&str>,
    ) -> bool {
        egui::Frame::NONE
            .fill(fill)
            .inner_margin(egui::Margin::symmetric(CELL_MARGIN_X, CELL_MARGIN_Y))
            .show(ui, |ui| {
                ui.set_min_width(LINE_NUMBER_WIDTH);
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(LINE_NUMBER_WIDTH, ROW_HEIGHT),
                    egui::Sense::click(),
                );
                if ui.is_rect_visible(rect) {
                    Self::show_icon(ui, rect, expanded, label.is_some());
                    if let Some(text) = label {
                        Self::show_label(ui, rect, text, palette);
                    }
                }
                response.clicked()
            })
            .inner
    }

    fn show_icon(ui: &mut egui::Ui, rect: egui::Rect, expanded: bool, align_left: bool) {
        let (top_icon, bottom_icon) = Self::icons(expanded);
        let icon_size = egui::vec2(COLLAPSE_ICON_WIDTH, COLLAPSE_ICON_HEIGHT);
        let center = if align_left {
            egui::pos2(
                rect.left() + COLLAPSE_ICON_LEFT_PADDING + icon_size.x / 2.0,
                rect.center().y,
            )
        } else {
            rect.center()
        };
        let top_rect = egui::Rect::from_center_size(
            egui::pos2(center.x, center.y - COLLAPSE_ICON_OFFSET_Y),
            icon_size,
        );
        let bottom_rect = egui::Rect::from_center_size(
            egui::pos2(center.x, center.y + COLLAPSE_ICON_OFFSET_Y),
            icon_size,
        );

        ui.put(top_rect, Self::image(ui, top_icon, icon_size));
        ui.put(bottom_rect, Self::image(ui, bottom_icon, icon_size));
    }

    fn show_label(ui: &egui::Ui, rect: egui::Rect, text: &str, palette: &DiffViewerPalette) {
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        ui.painter().text(
            egui::pos2(rect.right(), rect.center().y),
            egui::Align2::RIGHT_CENTER,
            text,
            font_id,
            palette.secondary_text,
        );
    }

    fn image(ui: &egui::Ui, icon: crate::Icon, icon_size: egui::Vec2) -> egui::Image<'static> {
        icon.ui_image(ui, crate::icon::IconSize::Small)
            .fit_to_exact_size(icon_size)
            .maintain_aspect_ratio(false)
    }

    fn icons(expanded: bool) -> (crate::Icon, crate::Icon) {
        if expanded {
            (crate::Icon::ChevronDown, crate::Icon::ChevronUp)
        } else {
            (crate::Icon::ChevronUp, crate::Icon::ChevronDown)
        }
    }
}
