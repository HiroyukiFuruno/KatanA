/* WHY: Verification of internal flags and state markers used to track font loading status. */

use super::*;
use egui::Context;

#[test]
fn test_setup_fonts_sets_fonts_loaded_flag() {
    let ctx = Context::default();
    let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
    SystemFontLoader::setup_fonts(&ctx, preset, None, None);

    let loaded = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("katana_fonts_loaded")));
    assert!(
        loaded.is_some(),
        "katana_fonts_loaded flag must always be set"
    );
}
