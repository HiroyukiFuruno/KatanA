#[cfg(target_os = "macos")]
use regex::Regex;
#[cfg(target_os = "macos")]
use resvg::{render, usvg};
#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(target_os = "macos")]
use tiny_skia::{Pixmap, PixmapPaint, Transform};

#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_PATH: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";
#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_FAMILY: &str = "Apple Color Emoji";
#[cfg(target_os = "macos")]
const MIN_EMOJI_PIXEL_SIZE: u32 = 16;
#[cfg(target_os = "macos")]
const EMOJI_RASTER_SCALE: u32 = 2;
#[cfg(target_os = "macos")]
const EMOJI_CANVAS_MULTIPLIER: u32 = 4;
#[cfg(target_os = "macos")]
const EMOJI_FONT_SIZE_RATIO: f32 = 1.0;
#[cfg(target_os = "macos")]
const EMOJI_CROP_PADDING: usize = 2;
#[cfg(target_os = "macos")]
const EMOJI_BASELINE_PADDING_RATIO: f32 = 0.2;
#[cfg(target_os = "macos")]
const EMOJI_TOP_PADDING_SHARE: f32 = 0.35;
#[cfg(target_os = "macos")]
const RGBA_CHANNEL_COUNT: usize = 4;
#[cfg(target_os = "macos")]
const RGBA_ALPHA_CHANNEL_OFFSET: usize = 3;

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EmojiCacheKey {
    grapheme: String,
    pixel_size: u32,
}

/// Rasterizes a single emoji grapheme into PNG bytes using Apple Color Emoji on macOS.
///
/// Returns `None` when the system font is unavailable or when the grapheme cannot be rasterized.
pub fn render_apple_color_emoji_png(grapheme: &str, pixel_size: u32) -> Option<Vec<u8>> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = grapheme;
        let _ = pixel_size;
        None
    }

    #[cfg(target_os = "macos")]
    {
        if grapheme.is_empty() || !Path::new(APPLE_COLOR_EMOJI_FONT_PATH).exists() {
            return None;
        }

        let key = EmojiCacheKey {
            grapheme: grapheme.to_owned(),
            pixel_size: pixel_size.max(MIN_EMOJI_PIXEL_SIZE),
        };

        let cache = emoji_png_cache();
        {
            let guard = cache.lock().expect("emoji raster cache lock poisoned");
            if let Some(bytes) = guard.get(&key) {
                return bytes.clone();
            }
        }

        let rendered = render_apple_color_emoji_png_uncached(&key.grapheme, key.pixel_size);
        cache
            .lock()
            .expect("emoji raster cache lock poisoned")
            .insert(key, rendered.clone());
        rendered
    }
}

/// Rewrites SVG `<text>` nodes containing emoji to prefer Apple Color Emoji on macOS.
///
/// This is used for badge services such as shields.io where emoji are embedded inside SVG text.
pub fn prefer_apple_color_emoji_in_svg(svg: &str) -> String {
    #[cfg(not(target_os = "macos"))]
    {
        svg.to_owned()
    }

    #[cfg(target_os = "macos")]
    {
        svg_text_regex()
            .replace_all(svg, |caps: &regex::Captures<'_>| {
                let attrs = caps.name("attrs").map_or("", |m| m.as_str());
                let text = caps.name("text").map_or("", |m| m.as_str());
                if !contains_emoji(text) {
                    caps.get(0)
                        .map_or_else(String::new, |full| full.as_str().to_owned())
                } else {
                    format!("<text{}>{}</text>", ensure_emoji_font_family(attrs), text)
                }
            })
            .into_owned()
    }
}

#[cfg(target_os = "macos")]
fn emoji_png_cache() -> &'static Mutex<HashMap<EmojiCacheKey, Option<Vec<u8>>>> {
    static EMOJI_PNG_CACHE: OnceLock<Mutex<HashMap<EmojiCacheKey, Option<Vec<u8>>>>> =
        OnceLock::new();
    EMOJI_PNG_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(target_os = "macos")]
fn render_apple_color_emoji_png_uncached(grapheme: &str, pixel_size: u32) -> Option<Vec<u8>> {
    let raster_size = pixel_size.max(MIN_EMOJI_PIXEL_SIZE) * EMOJI_RASTER_SCALE;
    let canvas_size = raster_size.saturating_mul(EMOJI_CANVAS_MULTIPLIER);
    let font_size = (raster_size as f32 * EMOJI_FONT_SIZE_RATIO).ceil();
    let midpoint = canvas_size as f32 / 2.0;
    let escaped = escape_svg_text(grapheme);
    let svg = format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{canvas}\" height=\"{canvas}\" ",
            "viewBox=\"0 0 {canvas} {canvas}\">",
            "<text x=\"{midpoint}\" y=\"{midpoint}\" text-anchor=\"middle\" ",
            "dominant-baseline=\"middle\" font-family=\"{font_family}\" ",
            "font-size=\"{font_size}px\">{escaped}</text></svg>"
        ),
        canvas = canvas_size,
        midpoint = midpoint,
        font_family = APPLE_COLOR_EMOJI_FONT_FAMILY,
        font_size = font_size,
        escaped = escaped,
    );

    let opts = usvg::Options {
        fontdb: emoji_font_db(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg, &opts).ok()?;
    let mut pixmap = Pixmap::new(canvas_size, canvas_size)?;
    render(&tree, Transform::identity(), &mut pixmap.as_mut());
    crop_non_transparent(&pixmap)?.encode_png().ok()
}

#[cfg(target_os = "macos")]
fn emoji_font_db() -> Arc<usvg::fontdb::Database> {
    static FONT_DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    Arc::clone(FONT_DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    }))
}

#[cfg(target_os = "macos")]
fn crop_non_transparent(pixmap: &Pixmap) -> Option<Pixmap> {
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    let mut found_opaque_pixel = false;

    for y in 0..height {
        for x in 0..width {
            let alpha =
                pixmap.data()[(y * width + x) * RGBA_CHANNEL_COUNT + RGBA_ALPHA_CHANNEL_OFFSET];
            if alpha > 0 {
                found_opaque_pixel = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if !found_opaque_pixel {
        return None;
    }

    let left = min_x.saturating_sub(EMOJI_CROP_PADDING);
    let top = min_y.saturating_sub(EMOJI_CROP_PADDING);
    let right = (max_x + EMOJI_CROP_PADDING).min(width - 1);
    let bottom = (max_y + EMOJI_CROP_PADDING).min(height - 1);
    let cropped_width = (right - left + 1) as u32;
    let cropped_height = (bottom - top + 1) as u32;
    let baseline_padding = ((cropped_height as f32) * EMOJI_BASELINE_PADDING_RATIO).round() as u32;
    let square_side = cropped_width.max(cropped_height.saturating_add(baseline_padding));
    let extra_x = (square_side - cropped_width) / 2;
    let extra_y = square_side - cropped_height;
    let extra_top = ((extra_y as f32) * EMOJI_TOP_PADDING_SHARE).round() as u32;
    let mut cropped = Pixmap::new(square_side, square_side)?;
    cropped.draw_pixmap(
        extra_x as i32 - left as i32,
        extra_top as i32 - top as i32,
        pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    Some(cropped)
}

#[cfg(target_os = "macos")]
fn escape_svg_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(target_os = "macos")]
fn svg_text_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?s)<text(?P<attrs>[^>]*)>(?P<text>.*?)</text>"#)
            .expect("valid svg text regex")
    })
}

#[cfg(target_os = "macos")]
fn font_family_double_quote_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"font-family\s*=\s*"(?P<family>[^"]*)""#)
            .expect("valid double-quoted font-family regex")
    })
}

#[cfg(target_os = "macos")]
fn font_family_single_quote_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"font-family\s*=\s*'(?P<family>[^']*)'"#)
            .expect("valid single-quoted font-family regex")
    })
}

#[cfg(target_os = "macos")]
fn ensure_emoji_font_family(attrs: &str) -> String {
    for (regex, quote) in [
        (font_family_double_quote_regex(), '"'),
        (font_family_single_quote_regex(), '\''),
    ] {
        if let Some(caps) = regex.captures(attrs) {
            let family = caps.name("family").map_or("", |m| m.as_str());
            if family.contains(APPLE_COLOR_EMOJI_FONT_FAMILY) {
                return attrs.to_owned();
            }

            let replacement = format!(
                "font-family={quote}{font_family}, {family}{quote}",
                font_family = APPLE_COLOR_EMOJI_FONT_FAMILY,
            );
            return regex.replace(attrs, replacement).into_owned();
        }
    }

    format!(r#"{attrs} font-family="{APPLE_COLOR_EMOJI_FONT_FAMILY}""#)
}

#[cfg(target_os = "macos")]
fn contains_emoji(text: &str) -> bool {
    text.chars().any(is_emoji_scalar)
}

#[cfg(target_os = "macos")]
fn is_emoji_scalar(ch: char) -> bool {
    matches!(
        ch as u32,
        0x00A9
            | 0x00AE
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2194..=0x21AA
            | 0x231A..=0x2328
            | 0x23CF
            | 0x23E9..=0x23FA
            | 0x24C2
            | 0x25AA..=0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2600..=0x27BF
            | 0x2934..=0x2935
            | 0x2B05..=0x2B55
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0x1F000..=0x1FAFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn apple_color_emoji_rasterizer_returns_png_bytes() {
        let bytes = render_apple_color_emoji_png("🌍", 24).expect("emoji png bytes");
        assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
        assert!(
            bytes.len() > 32,
            "png payload should not be trivially small"
        );
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn apple_color_emoji_rasterizer_safely_skips_non_macos() {
        assert!(render_apple_color_emoji_png("🌍", 24).is_none());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn prefer_apple_color_emoji_in_svg_prefixes_existing_font_family() {
        let svg = r#"<svg><text x="10" font-family="Verdana" font-size="14">❤️</text></svg>"#;

        let processed = prefer_apple_color_emoji_in_svg(svg);

        assert!(processed.contains(r#"font-family="Apple Color Emoji, Verdana""#));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn prefer_apple_color_emoji_in_svg_adds_font_family_when_missing() {
        let svg = r#"<svg><text x="10">❤️ Sponsor</text></svg>"#;

        let processed = prefer_apple_color_emoji_in_svg(svg);

        assert!(processed.contains(r#"font-family="Apple Color Emoji""#));
    }

    #[test]
    fn prefer_apple_color_emoji_in_svg_leaves_plain_text_unchanged() {
        let svg = r#"<svg><text x="10" font-family="Verdana">Sponsor</text></svg>"#;

        let processed = prefer_apple_color_emoji_in_svg(svg);

        assert_eq!(processed, svg);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn render_apple_color_emoji_returns_none_for_empty_grapheme() {
        assert!(render_apple_color_emoji_png("", 24).is_none());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn prefer_apple_color_emoji_in_svg_preserves_existing_emoji_font_family() {
        let svg = r#"<svg><text x="10" font-family="Apple Color Emoji, Verdana">❤️</text></svg>"#;
        let processed = prefer_apple_color_emoji_in_svg(svg);
        // Already has Apple Color Emoji — should not duplicate
        assert!(processed.contains(r#"font-family="Apple Color Emoji, Verdana""#));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn is_emoji_scalar_covers_supplemental_arrows_range() {
        // 0x2934 = ⤴ (ARROW POINTING RIGHTWARDS THEN CURVING UPWARDS)
        assert!(is_emoji_scalar('\u{2934}'));
        assert!(is_emoji_scalar('\u{2935}'));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn is_emoji_scalar_covers_misc_symbols_range() {
        // 0x2B05 = ⬅ (LEFTWARDS BLACK ARROW)
        assert!(is_emoji_scalar('\u{2B05}'));
        assert!(is_emoji_scalar('\u{2B55}'));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn is_emoji_scalar_covers_emoticons_range() {
        // 0x1F000..=0x1FAFF — Mahjong Tiles through Symbols Extended-A
        assert!(is_emoji_scalar('\u{1F600}')); // grinning face
        assert!(is_emoji_scalar('\u{1F000}')); // mahjong tile
    }
}
