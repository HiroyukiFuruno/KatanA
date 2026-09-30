/* WHY: Verification that the built-in emoji fallback survives excluding giant system fonts. */

use super::*;
use egui::FontId;

#[test]
fn test_builtin_emoji_fallback_renders_without_system_payload() {
    let ctx = egui::Context::default();
    ctx.set_fonts(FontDefinitions::default());

    let mut glyph = None;
    crate::test_ui::TestUiOps::run(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                "🌍".to_owned(),
                FontId::new(24.0, FontFamily::Proportional),
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
        "the built-in emoji fallback should rasterize a visible glyph"
    );
}
