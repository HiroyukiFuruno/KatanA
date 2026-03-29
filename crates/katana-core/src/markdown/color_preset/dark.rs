use super::DiagramColorPreset;
use std::sync::OnceLock;

pub fn get_dark_preset() -> &'static DiagramColorPreset {
    static DARK: OnceLock<DiagramColorPreset> = OnceLock::new();
    DARK.get_or_init(|| DiagramColorPreset {
        background: "transparent",
        text: "#E0E0E0",
        fill: "#2D2D2D",
        stroke: "#888888",
        arrow: "#AAAAAA",
        drawio_label_color: "#1A1A1A",
        mermaid_theme: "dark",
        plantuml_class_bg: "#2D2D2D",
        plantuml_note_bg: "#3A3A3A",
        plantuml_note_text: "#E0E0E0",
        syntax_theme_dark: "base16-ocean.dark",
        syntax_theme_light: "base16-ocean.light",
        preview_text: "#E0E0E0",
        proportional_font_candidates: vec![
            // WHY: macOS — Hiragino Sans (high-quality CJK + Latin rendering)
            "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/AquaKana.ttc",
            // WHY: Windows — Yu Gothic UI / Meiryo (CJK + Latin)
            "C:/Windows/Fonts/YuGothR.ttc",
            "C:/Windows/Fonts/yugothic.ttf",
            "C:/Windows/Fonts/meiryo.ttc",
            "C:/Windows/Fonts/segoeui.ttf",
            // WHY: Linux — Noto Sans (widely available via distro packages)
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ],
        monospace_font_candidates: vec![
            // WHY: macOS — Menlo (standard monospace since OS X 10.6)
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/SFMono-Regular.otf",
            "/System/Library/Fonts/Monaco.ttf",
            // WHY: Windows — Consolas (standard monospace since Vista)
            "C:/Windows/Fonts/consola.ttf",
            "C:/Windows/Fonts/cour.ttf",
            // WHY: Linux — standard monospace fonts
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        ],
        emoji_font_candidates: vec![
            // WHY: macOS — Apple Color Emoji
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            // WHY: Windows — Segoe UI Emoji (standard since Windows 8.1)
            "C:/Windows/Fonts/seguiemj.ttf",
            // WHY: Linux — Noto Color Emoji (widely available via distro packages)
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
        ],
        editor_font_size: DiagramColorPreset::DEFAULT_EDITOR_FONT_SIZE,
    })
}
