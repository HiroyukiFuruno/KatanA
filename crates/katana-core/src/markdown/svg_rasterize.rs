/* WHY: SVG rasterization utility.
Uses `resvg` + `usvg` to convert SVG text to an RGBA pixel buffer.
Returns the result as raw bytes compatible with egui's `ColorImage`. */

use resvg::{render, usvg};
use tiny_skia::Pixmap;

const MAX_RASTERIZED_SVG_EDGE: f32 = 8192.0;
const LIGHT_DARK_FUNCTION: &str = "light-dark(";

#[derive(Debug, Clone)]
pub struct RasterizedSvg {
    pub width: u32,
    pub height: u32,
    pub display_width: f32,
    pub display_height: f32,
    pub rgba: Vec<u8>,
}

pub struct SvgRasterizeOps;

impl SvgRasterizeOps {
    pub fn preprocess_for_rasterizer(svg_text: &str) -> String {
        let with_xml_entities = normalize_html_entities_for_xml(svg_text);
        let without_foreign_objects = strip_foreign_objects(&with_xml_entities);
        resolve_light_dark_functions(&without_foreign_objects)
    }

    pub fn rasterize_svg(svg_text: &str, scale: f32) -> Result<RasterizedSvg, SvgRasterizeError> {
        let opts = usvg::Options {
            /* WHY: Text inside SVG becomes invisible if system fonts are not provided. */
            fontdb: font_db(),
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

fn normalize_html_entities_for_xml(svg_text: &str) -> String {
    svg_text.replace("&nbsp;", "&#160;")
}

fn strip_foreign_objects(svg_text: &str) -> String {
    foreign_object_pattern()
        .replace_all(svg_text, "")
        .to_string()
}

fn foreign_object_pattern() -> &'static regex::Regex {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    PATTERN.get_or_init(|| {
        regex::Regex::new(concat!(
            r"(?is)<foreignObject\b[^>]*/>|",
            r"<foreignObject\b[^>]*>.*?</foreignObject>"
        ))
        .expect("valid foreignObject regex")
    })
}

fn resolve_light_dark_functions(svg_text: &str) -> String {
    let mut result = String::with_capacity(svg_text.len());
    let mut remaining = svg_text;
    while let Some(start) = find_light_dark_function(remaining) {
        let content_start = start + LIGHT_DARK_FUNCTION.len();
        result.push_str(&remaining[..start]);
        let Some((content_end, light_color)) =
            parse_light_dark_function(&remaining[content_start..])
        else {
            result.push_str(&remaining[start..content_start]);
            remaining = &remaining[content_start..];
            continue;
        };
        result.push_str(light_color.trim());
        remaining = &remaining[content_start + content_end + 1..];
    }
    result.push_str(remaining);
    result
}

fn find_light_dark_function(text: &str) -> Option<usize> {
    text.to_ascii_lowercase().find(LIGHT_DARK_FUNCTION)
}

fn parse_light_dark_function(content: &str) -> Option<(usize, &str)> {
    let mut depth = 0usize;
    let mut comma = None;
    for (index, character) in content.char_indices() {
        match character {
            '(' => depth += 1,
            ')' if depth == 0 => return comma.map(|comma_index| (index, &content[..comma_index])),
            ')' => depth -= 1,
            ',' if depth == 0 && comma.is_none() => comma = Some(index),
            _ => {}
        }
    }
    None
}

fn font_db() -> std::sync::Arc<usvg::fontdb::Database> {
    static FONT_DB: std::sync::OnceLock<std::sync::Arc<usvg::fontdb::Database>> =
        std::sync::OnceLock::new();
    std::sync::Arc::clone(FONT_DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        std::sync::Arc::new(db)
    }))
}

fn effective_scale(width: f32, height: f32, requested_scale: f32) -> f32 {
    let positive_scale = requested_scale.max(f32::MIN_POSITIVE);
    let width_scale = MAX_RASTERIZED_SVG_EDGE / width.max(1.0);
    let height_scale = MAX_RASTERIZED_SVG_EDGE / height.max(1.0);
    positive_scale.min(width_scale).min(height_scale)
}

#[derive(Debug, thiserror::Error)]
pub enum SvgRasterizeError {
    #[error("Failed to parse SVG: {0}")]
    ParseFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_reuses_invalid_light_dark_syntax() {
        let source = "<svg>fill: light-dark(red)</svg>";
        let output = SvgRasterizeOps::preprocess_for_rasterizer(source);
        assert_eq!(output, source);
    }

    #[test]
    fn parse_light_dark_function_without_comma_returns_none() {
        assert_eq!(parse_light_dark_function("red)"), None);
        assert_eq!(parse_light_dark_function(""), None);
    }

    #[test]
    fn parse_light_dark_function_with_nested_args() {
        assert_eq!(
            parse_light_dark_function("calc(1,2),ok)"),
            Some((12, "calc(1,2)"))
        );
    }
}
