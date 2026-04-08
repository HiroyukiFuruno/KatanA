use egui::{Context, FontFamily};

mod helpers;
mod normalize;
mod types;

pub use types::{NormalizeFonts, SystemFontLoader};

use normalize::{
    MARKDOWN_PROPORTIONAL_Y_OFFSET_FACTOR, MONO_PRIMARY_Y_OFFSET_FACTOR,
    PROPORTIONAL_Y_OFFSET_FACTOR,
};

impl SystemFontLoader {
    pub fn setup_fonts(
        ctx: &Context,
        preset: &katana_core::markdown::color_preset::DiagramColorPreset,
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
        emoji_candidates: &[&str],
        custom_font_path: Option<&str>,
        custom_font_name: Option<&str>,
    ) -> NormalizeFonts {
        let mut fonts = egui::FontDefinitions::default();

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

        let emoji_name = Self::load_first_valid(&mut fonts, emoji_candidates, None, "");
        if let Some(name) = &emoji_name {
            Self::append_fallback(&mut fonts, FontFamily::Proportional, name);
            Self::append_fallback(&mut fonts, FontFamily::Monospace, name);
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
}

#[cfg(test)]
mod tests;
