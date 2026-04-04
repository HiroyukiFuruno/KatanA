use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Default)]
pub struct DiagramColorPreset {
    pub background: &'static str,
    pub text: &'static str,
    pub fill: &'static str,
    pub stroke: &'static str,
    pub arrow: &'static str,
    pub drawio_label_color: &'static str,
    pub mermaid_theme: &'static str,
    pub plantuml_class_bg: &'static str,
    pub plantuml_note_bg: &'static str,
    pub plantuml_note_text: &'static str,
    pub syntax_theme_dark: &'static str,
    pub syntax_theme_light: &'static str,
    pub preview_text: &'static str,
    pub proportional_font_candidates: Vec<&'static str>,
    pub monospace_font_candidates: Vec<&'static str>,
    pub emoji_font_candidates: Vec<&'static str>,
    pub editor_font_size: f32,
}

pub static DARK_MODE: AtomicBool = AtomicBool::new(true);
