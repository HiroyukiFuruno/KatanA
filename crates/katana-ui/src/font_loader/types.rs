use egui::FontDefinitions;

pub struct NormalizeFonts {
    pub(crate) fonts: FontDefinitions,
    pub(crate) is_normalized: bool,
}

pub struct SystemFontLoader;
