use super::DiagramColorPreset;
use std::sync::OnceLock;

pub struct LightOps;

impl LightOps {
    pub fn get() -> &'static DiagramColorPreset {
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
            proportional_font_candidates: DiagramColorPreset::default_proportional_fonts(),
            monospace_font_candidates: DiagramColorPreset::default_monospace_fonts(),
            emoji_font_candidates: DiagramColorPreset::default_emoji_fonts(),
            editor_font_size: DiagramColorPreset::DEFAULT_EDITOR_FONT_SIZE,
        })
    }
}
