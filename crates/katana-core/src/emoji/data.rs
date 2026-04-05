use super::types::EmojiDataOps;

impl EmojiDataOps {
    #[cfg(target_os = "macos")]
    pub fn is_emoji_scalar(ch: char) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn is_emoji_scalar_covers_supplemental_arrows_range() {
        // WHY: 0x2934 = ⤴ (ARROW POINTING RIGHTWARDS THEN CURVING UPWARDS)
        assert!(EmojiDataOps::is_emoji_scalar('\u{2934}'));
        assert!(EmojiDataOps::is_emoji_scalar('\u{2935}'));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn is_emoji_scalar_covers_misc_symbols_range() {
        // WHY: 0x2B05 = ⬅ (LEFTWARDS BLACK ARROW)
        assert!(EmojiDataOps::is_emoji_scalar('\u{2B05}'));
        assert!(EmojiDataOps::is_emoji_scalar('\u{2B55}'));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn is_emoji_scalar_covers_emoticons_range() {
        // WHY: 0x1F000..=0x1FAFF — Mahjong Tiles through Symbols Extended-A
        assert!(EmojiDataOps::is_emoji_scalar('\u{1F600}'));
        assert!(EmojiDataOps::is_emoji_scalar('\u{1F000}'));
    }
}
