/* WHY: Cache rendered emoji PNGs in memory to avoid the overhead of repeatedly rendering the same grapheme at the same size. */

#[cfg(target_os = "macos")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmojiCacheKey {
    pub grapheme: String,
    pub pixel_size: u32,
}

#[cfg(target_os = "macos")]
pub struct EmojiCacheEntry {
    pub key: EmojiCacheKey,
    pub png: Option<Vec<u8>>,
}

#[cfg(target_os = "macos")]
pub struct EmojiCacheOps;

#[cfg(target_os = "macos")]
impl EmojiCacheOps {
    pub fn check_emoji_cache(key: &EmojiCacheKey) -> Option<Option<Vec<u8>>> {
        emoji_png_cache()
            .lock()
            .expect("lock")
            .iter()
            .find(|e| &e.key == key)
            .map(|e| e.png.clone())
    }

    pub fn store_emoji_cache(key: EmojiCacheKey, png: Option<Vec<u8>>) {
        emoji_png_cache()
            .lock()
            .expect("lock")
            .push(EmojiCacheEntry { key, png });
    }
}

#[cfg(target_os = "macos")]
fn emoji_png_cache() -> &'static Mutex<Vec<EmojiCacheEntry>> {
    static EMOJI_PNG_CACHE: OnceLock<Mutex<Vec<EmojiCacheEntry>>> = OnceLock::new();
    EMOJI_PNG_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}
