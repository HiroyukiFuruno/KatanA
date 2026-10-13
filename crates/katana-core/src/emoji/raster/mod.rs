/* WHY: Root module for emoji rasterization. Provides the high-level API for rendering emojis on macOS while delegating specifics to sub-modules. */

#[cfg(target_os = "macos")]
mod cache;
#[cfg(target_os = "macos")]
mod constants;
#[cfg(target_os = "macos")]
mod rendering;

use super::types::EmojiRasterOps;

#[cfg(target_os = "macos")]
use constants::*;

impl EmojiRasterOps {
    #[cfg(not(target_os = "macos"))]
    pub fn render_apple_color_emoji_png(_grapheme: &str, _pixel_size: u32) -> Option<Vec<u8>> {
        None
    }

    #[cfg(target_os = "macos")]
    pub fn render_apple_color_emoji_png(grapheme: &str, pixel_size: u32) -> Option<Vec<u8>> {
        if grapheme.is_empty() || !std::path::Path::new(APPLE_COLOR_EMOJI_FONT_PATH).exists() {
            return None;
        }
        emojis::get(grapheme)?;
        let key = cache::EmojiCacheKey {
            grapheme: grapheme.to_owned(),
            pixel_size: pixel_size.max(MIN_EMOJI_PIXEL_SIZE),
        };

        if let Some(cached) = cache::EmojiCacheOps::check_emoji_cache(&key) {
            return cached;
        }

        let rendered = rendering::EmojiRenderingOps::render_apple_color_emoji_png_uncached(
            &key.grapheme,
            key.pixel_size,
        );
        cache::EmojiCacheOps::store_emoji_cache(key, rendered.clone());
        rendered
    }
}
