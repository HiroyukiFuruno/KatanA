use std::fs;

use egui::{FontData, FontDefinitions, FontFamily};

use super::types::SystemFontLoader;

impl SystemFontLoader {
    pub(super) fn load_first_valid(
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

    pub(super) fn prepend_primary(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, name.to_string());
        }
    }

    pub(super) fn append_fallback(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push(name.to_string());
        }
    }

    pub(super) fn insert_after_primary(
        fonts: &mut FontDefinitions,
        family: FontFamily,
        name: &str,
    ) {
        if let Some(list) = fonts.families.get_mut(&family) {
            let pos = 1.min(list.len());
            list.insert(pos, name.to_string());
        }
    }

    pub(super) fn inject_custom_font(fonts: &mut FontDefinitions, path: &str, name: &str) {
        let Ok(data) = fs::read(path) else { return };
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(FontData::from_owned(data)),
        );
        Self::prepend_primary(fonts, FontFamily::Proportional, name);
    }
}
