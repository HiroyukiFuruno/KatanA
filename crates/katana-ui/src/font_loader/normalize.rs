use egui::{FontDefinitions, FontFamily};

use super::types::{NormalizeFonts, SystemFontLoader};

const MONO_FALLBACK_Y_OFFSET_FACTOR: f32 = 0.40;
pub(super) const MONO_PRIMARY_Y_OFFSET_FACTOR: f32 = -0.15;
pub(super) const MARKDOWN_PROPORTIONAL_Y_OFFSET_FACTOR: f32 = 0.0;
/* WHY: Proportional font descender space causes text to appear ~3px above visual center. */
pub(super) const PROPORTIONAL_Y_OFFSET_FACTOR: f32 = 0.25;

#[cfg(target_os = "linux")]
pub(super) const LINUX_Y_OFFSET: f32 = -2.5;
#[cfg(not(target_os = "linux"))]
pub(super) const LINUX_Y_OFFSET: f32 = 0.0;

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
            y_offset: LINUX_Y_OFFSET,
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
