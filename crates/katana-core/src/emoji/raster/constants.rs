/* WHY: Centralized constants for emoji rasterization to ensure consistent sizing and alignment. */

#[cfg(target_os = "macos")]
pub const APPLE_COLOR_EMOJI_FONT_PATH: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";
#[cfg(target_os = "macos")]
pub const APPLE_COLOR_EMOJI_FONT_FAMILY: &str = "Apple Color Emoji";

#[cfg(target_os = "macos")]
pub const MIN_EMOJI_PIXEL_SIZE: u32 = 16;
#[cfg(target_os = "macos")]
pub const EMOJI_RASTER_SCALE: u32 = 2;
#[cfg(target_os = "macos")]
pub const EMOJI_CANVAS_MULTIPLIER: u32 = 4;
#[cfg(target_os = "macos")]
pub const EMOJI_FONT_SIZE_RATIO: f32 = 1.0;
#[cfg(target_os = "macos")]
pub const EMOJI_CROP_PADDING: usize = 2;
#[cfg(target_os = "macos")]
pub const EMOJI_BASELINE_PADDING_RATIO: f32 = 0.2;
#[cfg(target_os = "macos")]
pub const EMOJI_TOP_PADDING_SHARE: f32 = 0.35;

#[cfg(target_os = "macos")]
pub const RGBA_CHANNEL_COUNT: usize = 4;
#[cfg(target_os = "macos")]
pub const RGBA_ALPHA_CHANNEL_OFFSET: usize = 3;
