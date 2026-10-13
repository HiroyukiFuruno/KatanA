/* WHY: Specialized logic for inline text and emoji rendering to maintain modularity and isolation of presentation logic. */

use super::super::{
    EMOJI_INLINE_DISPLAY_SIZE, EMOJI_INLINE_NEGATIVE_SPACE, EMOJI_INLINE_UNDERLINE_OFFSET_Y,
};
use eframe::egui;

pub struct HtmlInlineTextOps;

impl HtmlInlineTextOps {
    pub(crate) fn render_emoji_with_underline(
        ui: &mut egui::Ui,
        grapheme: &str,
        bytes: Vec<u8>,
        tooltip: &str,
        clicked: &mut bool,
    ) {
        let uri = format!("emoji://{grapheme}");
        let mut response = ui.add(
            egui::Image::from_bytes(uri, bytes)
                .fit_to_exact_size(egui::vec2(
                    EMOJI_INLINE_DISPLAY_SIZE,
                    EMOJI_INLINE_DISPLAY_SIZE,
                ))
                .sense(egui::Sense::click()),
        );
        response = response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tooltip);
        let y = response.rect.max.y + EMOJI_INLINE_UNDERLINE_OFFSET_Y;
        ui.painter().hline(
            response.rect.x_range(),
            y,
            egui::Stroke::new(1.0, ui.visuals().hyperlink_color),
        );
        if response.clicked() {
            *clicked = true;
        }
        ui.add_space(EMOJI_INLINE_NEGATIVE_SPACE);
    }
}
