use super::DiagramColorPreset;
use std::sync::OnceLock;

pub fn get_light_preset() -> &'static DiagramColorPreset {
    static LIGHT: OnceLock<DiagramColorPreset> = OnceLock::new();
    LIGHT.get_or_init(|| DiagramColorPreset {
        background: "transparent",
        text: "#333333",
        fill: "#fff2cc",
        stroke: "#d6b656",
        arrow: "#555555",
        drawio_label_color: "#333333",
        mermaid_theme: "default",
        plantuml_class_bg: "#FEFECE",
        plantuml_note_bg: "#FBFB77",
        plantuml_note_text: "#333333",
        syntax_theme_dark: "base16-ocean.dark",
        syntax_theme_light: "InspiredGitHub",
        preview_text: "#333333",
        proportional_font_candidates: vec![
            // WHY: macOS
            "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/AquaKana.ttc",
            // WHY: Windows
            "C:/Windows/Fonts/YuGothR.ttc",
            "C:/Windows/Fonts/yugothic.ttf",
            "C:/Windows/Fonts/meiryo.ttc",
            "C:/Windows/Fonts/segoeui.ttf",
            // WHY: Linux
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ],
        monospace_font_candidates: vec![
            // WHY: macOS
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/SFMono-Regular.otf",
            "/System/Library/Fonts/Monaco.ttf",
            // WHY: Windows
            "C:/Windows/Fonts/consola.ttf",
            "C:/Windows/Fonts/cour.ttf",
            // WHY: Linux
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        ],
        emoji_font_candidates: vec![
            // WHY: macOS — Apple Color Emoji
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            // WHY: Windows
            "C:/Windows/Fonts/seguiemj.ttf",
            // WHY: Linux
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
        ],
        editor_font_size: DiagramColorPreset::DEFAULT_EDITOR_FONT_SIZE,
    })
}
