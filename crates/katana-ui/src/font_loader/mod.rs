use egui::{Context, FontData, FontDefinitions, FontFamily};
use katana_core::markdown::color_preset::DiagramColorPreset;
use std::fs;

const MONO_FALLBACK_Y_OFFSET_FACTOR: f32 = 0.40;
const MONO_PRIMARY_Y_OFFSET_FACTOR: f32 = -0.15;
const MARKDOWN_PROPORTIONAL_Y_OFFSET_FACTOR: f32 = 0.0;
// WHY: Proportional font descender space causes text to appear ~3px above visual center.
// This offset compensates for the unused descender area in CJK/UI text rendering.
const PROPORTIONAL_Y_OFFSET_FACTOR: f32 = 0.25;

mod types;
pub use types::{NormalizeFonts, SystemFontLoader};

impl NormalizeFonts {
    pub fn new(fonts: FontDefinitions) -> Self {
        Self {
            fonts,
            is_normalized: false,
        }
    }

    pub fn normalize(mut self, proportional_candidates: &[&str]) -> Self {
        if self.is_normalized {
            return self;
        }

        self.normalize_cjk_baseline(proportional_candidates);

        self.is_normalized = true;
        self
    }

    fn normalize_cjk_baseline(&mut self, proportional_candidates: &[&str]) {
        let tweaked_fallback = egui::FontTweak {
            coords: Default::default(),
            hinting_override: None,
            scale: 1.0,
            y_offset_factor: MONO_FALLBACK_Y_OFFSET_FACTOR + MONO_PRIMARY_Y_OFFSET_FACTOR,
            y_offset: 0.0,
        };
        let mono_fallback_name = SystemFontLoader::load_first_valid(
            &mut self.fonts,
            proportional_candidates,
            Some(tweaked_fallback),
            "_mono_fallback",
        );

        if let Some(name) = &mono_fallback_name {
            SystemFontLoader::insert_after_primary(&mut self.fonts, FontFamily::Monospace, name);
        }
    }

    pub fn is_normalized(&self) -> bool {
        self.is_normalized
    }

    pub fn fonts(&self) -> &FontDefinitions {
        &self.fonts
    }

    pub fn into_inner(self) -> FontDefinitions {
        self.fonts
    }
}

impl SystemFontLoader {
    pub fn setup_fonts(
        ctx: &Context,
        preset: &DiagramColorPreset,
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) {
        let normalized = Self::build_font_definitions(
            &preset.proportional_font_candidates,
            &preset.monospace_font_candidates,
            &preset.emoji_font_candidates,
            custom_font_path,
            custom_font_name,
        );
        let is_loaded = normalized
            .fonts
            .families
            .contains_key(&egui::FontFamily::Name("MarkdownProportional".into()));
        ctx.set_fonts(normalized.into_inner());
        let id = egui::Id::new("katana_fonts_loaded");
        ctx.data_mut(|d| d.insert_temp(id, is_loaded));

        #[cfg(debug_assertions)]
        ctx.global_style_mut(|style| {
            style.debug.debug_on_hover = false;
            style.debug.show_expand_width = false;
            style.debug.show_expand_height = false;
            style.debug.show_widget_hits = false;
        });
    }

    pub fn build_font_definitions(
        proportional_candidates: &[&str],
        monospace_candidates: &[&str],
        _emoji_candidates: &[&str],
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) -> NormalizeFonts {
        let mut fonts = FontDefinitions::default();

        let prop_tweak = egui::FontTweak {
            coords: Default::default(),
            hinting_override: None,
            scale: 1.0,
            y_offset_factor: PROPORTIONAL_Y_OFFSET_FACTOR,
            y_offset: 0.0,
        };
        let prop_name =
            Self::load_first_valid(&mut fonts, proportional_candidates, Some(prop_tweak), "");

        let markdown_tweak = egui::FontTweak {
            coords: Default::default(),
            hinting_override: None,
            scale: 1.0,
            y_offset_factor: MARKDOWN_PROPORTIONAL_Y_OFFSET_FACTOR,
            y_offset: 0.0,
        };
        let markdown_name = Self::load_first_valid(
            &mut fonts,
            proportional_candidates,
            Some(markdown_tweak),
            "_markdown",
        );

        let mono_tweak = egui::FontTweak {
            coords: Default::default(),
            hinting_override: None,
            scale: 1.0,
            y_offset_factor: MONO_PRIMARY_Y_OFFSET_FACTOR,
            y_offset: 0.0,
        };
        let mono_name =
            Self::load_first_valid(&mut fonts, monospace_candidates, Some(mono_tweak), "");

        if let Some(name) = &prop_name {
            Self::prepend_primary(&mut fonts, FontFamily::Proportional, name);
        }
        if let Some(name) = &mono_name {
            Self::prepend_primary(&mut fonts, FontFamily::Monospace, name);
        }
        if let Some(name) = &markdown_name {
            Self::prepend_primary(
                &mut fonts,
                FontFamily::Name("MarkdownProportional".into()),
                name,
            );
        }

        if let Some(name) = &mono_name {
            Self::append_fallback(&mut fonts, FontFamily::Proportional, name);
        }
        if let Some(name) = &mono_name {
            Self::append_fallback(
                &mut fonts,
                FontFamily::Name("MarkdownProportional".into()),
                name,
            );
        }

        if let (Some(path), Some(name)) = (custom_font_path, custom_font_name) {
            Self::inject_custom_font(&mut fonts, path, name);
        }

        NormalizeFonts::new(fonts).normalize(proportional_candidates)
    }

    fn load_first_valid(
        fonts: &mut FontDefinitions,
        candidates: &[&str],
        tweak: Option<egui::FontTweak>,
        suffix: &str,
    ) -> Option<String> {
        for &path in candidates {
            let Ok(data) = fs::read(path) else { continue };
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string()
                + suffix;

            let mut font_data = FontData::from_owned(data);
            if let Some(t) = tweak {
                font_data.tweak = t;
            }

            fonts
                .font_data
                .insert(name.clone(), std::sync::Arc::new(font_data));
            return Some(name);
        }
        None
    }

    fn prepend_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, name.to_string());
        }
    }

    fn append_fallback(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push(name.to_string());
        }
    }

    fn insert_after_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            let pos = 1.min(list.len());
            list.insert(pos, name.to_string());
        }
    }

    fn inject_custom_font(fonts: &mut FontDefinitions, path: &str, name: &str) {
        let Ok(data) = fs::read(path) else { return };
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(FontData::from_owned(data)),
        );
        Self::prepend_primary(fonts, FontFamily::Proportional, name);
    }
}

#[cfg(test)]
mod tests;
