use super::types::*;
use eframe::egui;
use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_platform::theme::{Rgb, Rgba, ThemeColors, ThemeMode};

const STROKE_NORMAL: f32 = 1.0;
const STROKE_MEDIUM: f32 = 1.5;
const STROKE_BOLD: f32 = 2.0;

const HEADING_SIZE_RATIO: f32 = 1.5;
const SMALL_SIZE_RATIO: f32 = 0.75;
const STRONG_BLEND_RATIO: f32 = 0.3;

impl ThemeBridgeOps {
    pub fn visuals_from_theme(colors: &ThemeColors) -> egui::Visuals {
        let dark = colors.mode == ThemeMode::Dark;

        let bg = Self::rgb_to_color32(colors.system.background);
        let panel_bg = Self::rgb_to_color32(colors.system.panel_background);
        let text = Self::rgb_to_color32(colors.system.text);
        let text_secondary = Self::rgb_to_color32(colors.system.text_secondary);
        let accent = Self::rgb_to_color32(colors.system.accent);
        let border = Self::rgb_to_color32(colors.system.border);
        let selection_bg = Self::rgb_to_color32(colors.system.selection);
        let highlight_bg = Self::rgba_to_color32(colors.system.active_file_highlight);
        let code_bg = Self::rgb_to_color32(colors.code.background);
        let warning = Self::rgb_to_color32(colors.system.warning_text);

        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.panel_fill = panel_bg;
        visuals.window_fill = bg;
        visuals.extreme_bg_color = code_bg;
        visuals.code_bg_color = code_bg;
        visuals.faint_bg_color = panel_bg;
        visuals.warn_fg_color = warning;

        visuals.selection.bg_fill = selection_bg;
        visuals.selection.stroke = egui::Stroke::new(STROKE_NORMAL, accent);

        visuals.override_text_color = None;
        visuals.widgets.noninteractive.bg_fill = crate::theme_bridge::TRANSPARENT;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text);
        // WHY: All bg_stroke.width values are STROKE_NORMAL (1.0) — identical for every state.
        // egui's inner_margin formula: button_padding + expansion - stroke.width.
        // With expansion=0 and uniform stroke.width=1.0, inner_margin = button_padding - 1.0
        // for ALL states (inactive, hovered, active). Content never shifts on hover.
        // Color is TRANSPARENT for noninteractive/inactive so the border is invisible
        // at rest, but the width is pre-reserved in the layout calculation.
        // WHY expansion=0: outer_margin = -expansion = 0, preventing the drawn frame
        // from overflowing the allocated rect (which causes visual "border inflation").
        visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(STROKE_NORMAL, crate::theme_bridge::INVISIBLE);
        visuals.widgets.noninteractive.expansion = 0.0;

        visuals.widgets.inactive.bg_fill = panel_bg;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text_secondary);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(STROKE_NORMAL, crate::theme_bridge::INVISIBLE);
        visuals.widgets.inactive.expansion = 0.0;

        visuals.widgets.hovered.bg_fill = highlight_bg;
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(STROKE_MEDIUM, accent);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(STROKE_NORMAL, accent);
        visuals.widgets.hovered.expansion = 0.0;

        let strong = strengthen_color(text, dark);
        visuals.widgets.active.bg_fill = accent;
        visuals.widgets.active.fg_stroke = egui::Stroke::new(STROKE_BOLD, strong);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(STROKE_NORMAL, accent);
        visuals.widgets.active.expansion = 0.0;

        visuals.widgets.open.bg_fill = panel_bg;
        visuals.widgets.open.fg_stroke = egui::Stroke::new(STROKE_NORMAL, text_secondary);
        visuals.widgets.open.bg_stroke = egui::Stroke::new(STROKE_NORMAL, border);
        visuals.widgets.open.expansion = 0.0;

        visuals
    }

    pub fn rgb_to_color32(c: Rgb) -> egui::Color32 {
        egui::Color32::from_rgb(c.r, c.g, c.b)
    }

    pub fn rgba_to_color32(c: Rgba) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    }

    pub fn apply_font_size(ctx: &egui::Context, font_size: f32) {
        ctx.global_style_mut(|style| {
            let heading = font_size * HEADING_SIZE_RATIO;
            let small = font_size * SMALL_SIZE_RATIO;
            for (text_style, font_id) in style.text_styles.iter_mut() {
                match text_style {
                    egui::TextStyle::Heading => font_id.size = heading,
                    egui::TextStyle::Small => font_id.size = small,
                    _ => font_id.size = font_size,
                }
            }
        });
    }

    pub fn apply_font_family(ctx: &egui::Context, family_name: &str) {
        tracing::debug!("apply_font_family: Start ({family_name})");
        let preset = DiagramColorPreset::current();
        let mut custom_path = None;
        let mut custom_name = None;

        let mut default_family = egui::FontFamily::Proportional;
        if family_name == "Proportional" {
            default_family = egui::FontFamily::Proportional;
        } else if family_name == "Monospace" {
            default_family = egui::FontFamily::Monospace;
        } else {
            let os_fonts = katana_platform::os_fonts::OsFontScanner::cached_fonts();
            if let Some((name, path)) = os_fonts.iter().find(|(name, _)| name == family_name) {
                custom_path = Some(path.as_str());
                custom_name = Some(name.as_str());
                default_family = egui::FontFamily::Proportional;
            }
        }

        crate::font_loader::SystemFontLoader::setup_fonts(ctx, preset, custom_path, custom_name);
        tracing::debug!("apply_font_family: End ({family_name})");

        ctx.global_style_mut(|style| {
            for (text_style, font_id) in style.text_styles.iter_mut() {
                if *text_style != egui::TextStyle::Monospace {
                    font_id.family = default_family.clone();
                }
            }
        });
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
        egui::Color32::from_rgb(r, g, b)
    }

    pub fn from_gray(l: u8) -> egui::Color32 {
        egui::Color32::from_gray(l)
    }

    pub fn from_black_alpha(a: u8) -> egui::Color32 {
        egui::Color32::from_black_alpha(a)
    }

    pub fn from_white_alpha(a: u8) -> egui::Color32 {
        egui::Color32::from_white_alpha(a)
    }

    pub fn from_rgba_unmultiplied(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    pub fn from_rgba_premultiplied(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
        egui::Color32::from_rgba_premultiplied(r, g, b, a)
    }
}

fn strengthen_color(base: egui::Color32, dark: bool) -> egui::Color32 {
    let target: egui::Color32 = if dark {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    };
    let lerp = |a: u8, b: u8| -> u8 {
        let a = f32::from(a);
        let b = f32::from(b);
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let result = (a + (b - a) * STRONG_BLEND_RATIO) as u8;
        result
    };
    egui::Color32::from_rgb(
        lerp(base.r(), target.r()),
        lerp(base.g(), target.g()),
        lerp(base.b(), target.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_platform::theme::{ThemeMode, ThemePreset};

    #[test]
    fn test_apply_font_family_proportional() {
        let ctx = egui::Context::default();
        ThemeBridgeOps::apply_font_family(&ctx, "Proportional");
        let style = ctx.global_style();
        assert_eq!(
            style
                .text_styles
                .get(&egui::TextStyle::Body)
                .unwrap()
                .family,
            egui::FontFamily::Proportional
        );
    }

    #[test]
    fn test_apply_font_family_monospace() {
        let ctx = egui::Context::default();
        ThemeBridgeOps::apply_font_family(&ctx, "Monospace");
        let style = ctx.global_style();
        assert_eq!(
            style
                .text_styles
                .get(&egui::TextStyle::Body)
                .unwrap()
                .family,
            egui::FontFamily::Monospace
        );
    }

    #[test]
    fn visuals_from_theme_light_mode_uses_light_base() {
        let colors = ThemePreset::KatanaLight.colors();
        assert_eq!(colors.mode, ThemeMode::Light);
        let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
        assert!(!visuals.dark_mode);
    }

    #[test]
    fn visuals_from_theme_dark_mode_uses_dark_base() {
        let colors = ThemePreset::KatanaDark.colors();
        assert_eq!(colors.mode, ThemeMode::Dark);
        let visuals = ThemeBridgeOps::visuals_from_theme(&colors);
        assert!(visuals.dark_mode);
    }

    #[test]
    fn strengthen_color_darkens_in_light_mode() {
        let base = egui::Color32::from_rgb(200, 200, 200);
        let result = strengthen_color(base, false);
        assert!(result.r() < base.r());
        assert!(result.g() < base.g());
        assert!(result.b() < base.b());
    }

    #[test]
    fn test_color_helpers() {
        assert_eq!(
            ThemeBridgeOps::from_rgb(255, 0, 0),
            egui::Color32::from_rgb(255, 0, 0)
        );
        assert_eq!(
            ThemeBridgeOps::from_gray(128),
            egui::Color32::from_gray(128)
        );
        assert_eq!(
            ThemeBridgeOps::from_black_alpha(128),
            egui::Color32::from_black_alpha(128)
        );
    }
}
