/* WHY: Verification of Apple Color Emoji rasterization and layout. */

use super::*;
#[cfg(target_os = "macos")]
use egui::{FontData, FontId};
#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "macos")]
use std::sync::Arc;

#[test]
#[cfg(target_os = "macos")]
fn test_apple_color_emoji_family_renders_directly() {
    let data = fs::read("/System/Library/Fonts/Apple Color Emoji.ttc").expect("apple emoji font");
    let mut fonts = FontDefinitions::empty();
    fonts.font_data.insert(
        APPLE_COLOR_EMOJI_FONT_NAME.to_string(),
        Arc::new(FontData::from_owned(data)),
    );
    fonts.families.insert(
        FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into()),
        vec![APPLE_COLOR_EMOJI_FONT_NAME.to_string()],
    );

    let ctx = egui::Context::default();
    ctx.set_fonts(fonts);

    let mut glyph = None;
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                "🌍".to_owned(),
                FontId::new(24.0, FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into())),
                egui::Color32::WHITE,
            );
            glyph = galley
                .rows
                .first()
                .and_then(|row| row.glyphs.first())
                .copied();
        });
    });

    let glyph = glyph.expect("emoji glyph should be laid out");
    assert!(
        !glyph.uv_rect.is_nothing(),
        "Apple Color Emoji should rasterize a visible glyph when used directly"
    );
}
