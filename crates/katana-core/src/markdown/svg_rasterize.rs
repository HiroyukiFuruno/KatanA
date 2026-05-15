/* WHY: SVG rasterization utility.
Uses `resvg` + `usvg` to convert SVG text to an RGBA pixel buffer.
Returns the result as raw bytes compatible with egui's `ColorImage`. */

mod preprocess;
mod types;

pub use types::*;

use resvg::{render, usvg};
use tiny_skia::Pixmap;

const MAX_RASTERIZED_SVG_EDGE: f32 = 8192.0;
const FALLBACK_PROPORTIONAL_FONT_FAMILY: &str = "Ubuntu";
const FALLBACK_MONOSPACE_FONT_FAMILY: &str = "Hack";

impl SvgRasterizeOps {
    pub fn preprocess_for_rasterizer(svg_text: &str) -> String {
        preprocess::preprocess_for_rasterizer(svg_text)
    }

    pub fn rasterize_svg(svg_text: &str, scale: f32) -> Result<RasterizedSvg, SvgRasterizeError> {
        let opts = usvg::Options {
            /* WHY: Text inside SVG becomes invisible if system fonts are not provided. */
            fontdb: font_db(),
            font_family: FALLBACK_PROPORTIONAL_FONT_FAMILY.to_string(),
            ..usvg::Options::default()
        };
        let compatible_svg = Self::preprocess_for_rasterizer(svg_text);
        let tree = usvg::Tree::from_str(&compatible_svg, &opts)
            .map_err(|e| SvgRasterizeError::ParseFailed(e.to_string()))?;
        let size = tree.size();
        let display_width = size.width();
        let display_height = size.height();
        let effective_scale = effective_scale(display_width, display_height, scale);
        let width = ((display_width * effective_scale).ceil() as u32).max(1);
        let height = ((display_height * effective_scale).ceil() as u32).max(1);
        /* WHY: `Pixmap::new` is always `Some` because `max(1)` guarantees width/height >= 1. */
        let mut pixmap =
            Pixmap::new(width, height).expect("BUG: width/height >= 1 guaranteed by max(1)");
        /* WHY: Start with a transparent canvas. Each diagram renderer is responsible
        for setting the correct background via the DiagramColorPreset:
          - PlantUML: `skinparam backgroundColor transparent`
          - DrawIo:   SVG has no background rect — transparent by default
          - Mermaid:  emits SVG with theme-managed transparent background
        The transparent base lets diagram content blend naturally with the
        host application's dark/light theme. */
        let transform = tiny_skia::Transform::from_scale(effective_scale, effective_scale);
        render(&tree, transform, &mut pixmap.as_mut());
        Ok(RasterizedSvg {
            width,
            height,
            display_width,
            display_height,
            rgba: pixmap.take(),
        })
    }
}

fn font_db() -> std::sync::Arc<usvg::fontdb::Database> {
    static FONT_DB: std::sync::OnceLock<std::sync::Arc<usvg::fontdb::Database>> =
        std::sync::OnceLock::new();
    std::sync::Arc::clone(FONT_DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        load_embedded_fonts(&mut db);
        set_generic_font_families(&mut db);
        std::sync::Arc::new(db)
    }))
}

fn load_embedded_fonts(db: &mut usvg::fontdb::Database) {
    db.load_font_data(epaint_default_fonts::UBUNTU_LIGHT.to_vec());
    db.load_font_data(epaint_default_fonts::HACK_REGULAR.to_vec());
    db.load_font_data(epaint_default_fonts::NOTO_EMOJI_REGULAR.to_vec());
    db.load_font_data(epaint_default_fonts::EMOJI_ICON.to_vec());
}

fn set_generic_font_families(db: &mut usvg::fontdb::Database) {
    db.set_serif_family(FALLBACK_PROPORTIONAL_FONT_FAMILY);
    db.set_sans_serif_family(FALLBACK_PROPORTIONAL_FONT_FAMILY);
    db.set_cursive_family(FALLBACK_PROPORTIONAL_FONT_FAMILY);
    db.set_fantasy_family(FALLBACK_PROPORTIONAL_FONT_FAMILY);
    db.set_monospace_family(FALLBACK_MONOSPACE_FONT_FAMILY);
}

fn effective_scale(width: f32, height: f32, requested_scale: f32) -> f32 {
    let positive_scale = requested_scale.max(f32::MIN_POSITIVE);
    let width_scale = MAX_RASTERIZED_SVG_EDGE / width.max(1.0);
    let height_scale = MAX_RASTERIZED_SVG_EDGE / height.max(1.0);
    positive_scale.min(width_scale).min(height_scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_handles_plantuml_processing_instructions() {
        let source = concat!(
            r#"<?plantuml 1.2026.2?>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20px" height="20px">"#,
            r##"<g><?plantuml-src abc?><rect width="20" height="20" fill="#2D2D2D"/>"##,
            r##"<text x="2" y="12" fill="#E0E0E0">PUML</text></g></svg>"##
        );

        let rasterized = SvgRasterizeOps::rasterize_svg(source, 1.0).expect("rasterize PlantUML");

        assert!(rasterized.rgba.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }
}
