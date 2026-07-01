use eframe::egui;

/// Key cap rendering constants
const KEY_CAP_ROUNDING: f32 = 4.0;
/// Shared horizontal padding for ALL key caps.
const KEY_CAP_MARGIN_X: f32 = 4.0;
/// Shared vertical padding for ALL key caps.
const KEY_CAP_MARGIN_Y: f32 = 1.9;
const KEY_CAP_FONT_SIZE: f32 = 11.0;
/// Single-char keys and modifier icons use a fixed inner rect, slightly wider than tall
/// to match the natural proportions of physical key caps (reference image).
/// Long keys (Esc, Backspace…) share the same height via KEY_CAP_MARGIN_Y; width is auto.
const KEY_CAP_FIXED_W: f32 = 16.0;
const KEY_CAP_FIXED_H: f32 = 16.0;
const STROKE_FALLBACK_OPACITY: f32 = 0.3;
const STROKE_WIDTH: f32 = 1.0;
/// Minimum alpha to consider a stroke/fill color "visible".
/// The theme uses INVISIBLE (alpha=1/255) for bg_stroke, so any alpha
/// below this threshold should trigger the fallback.
const MIN_VISIBLE_ALPHA: u8 = 10;
const BADGE_BACKGROUND_MULTIPLIER: f32 = 0.06;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ShortcutDisplayPlatform {
    Macos,
    NonMacos,
}

impl ShortcutDisplayPlatform {
    fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::NonMacos
        }
    }
}

pub struct KeyCapOps;

impl KeyCapOps {
    /* WHY: Renders one key-cap badge.
    Design rules:
      - 1-char keys and modifier icons → exact square (KEY_CAP_FIXED_SIZE × KEY_CAP_FIXED_SIZE inner)
        Content is centered via add_sized, which is the correct egui idiom.
      - Long keys (Esc, Backspace…) → same vertical padding, auto width.
    Background and stroke are derived explicitly so the badge is always visible regardless of theme. */
    pub fn draw_key_cap(ui: &mut egui::Ui, token: &str) {
        let text_color = ui.visuals().text_color();

        /* WHY: The theme's code_bg_color can be transparent or identical to the panel background.
        Derive a visible badge colour from text_color to always look like a raised key. */
        let mut bg = ui.visuals().code_bg_color;
        if bg.a() < MIN_VISIBLE_ALPHA
            || bg == ui.visuals().window_fill
            || bg == ui.visuals().panel_fill
        {
            bg = text_color.linear_multiply(BADGE_BACKGROUND_MULTIPLIER);
        }

        let lower = token.to_lowercase();
        let platform = ShortcutDisplayPlatform::current();

        let icon_opt = Self::modifier_icon(&lower, platform);

        let mut stroke = ui.visuals().widgets.inactive.bg_stroke;
        if stroke.width == 0.0 || stroke.color.a() < MIN_VISIBLE_ALPHA {
            stroke = egui::Stroke::new(
                STROKE_WIDTH,
                text_color.linear_multiply(STROKE_FALLBACK_OPACITY),
            );
        }

        let mut frame = egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(
                KEY_CAP_MARGIN_X as i8,
                KEY_CAP_MARGIN_Y as i8,
            ))
            .corner_radius(KEY_CAP_ROUNDING)
            .fill(bg);
        frame.stroke = stroke;

        /* Single-char / modifier icon → fixed square badge */
        if token.chars().count() == 1 || icon_opt.is_some() {
            frame.show(ui, |ui| {
                let fixed = egui::vec2(KEY_CAP_FIXED_W, KEY_CAP_FIXED_H);
                if let Some(icon) = icon_opt {
                    /* add_sized allocates exactly `fixed` and centers the widget inside. */
                    let image = icon
                        .ui_image(ui, crate::icon::IconSize::Small)
                        .tint(text_color);
                    ui.add_sized(fixed, image);
                } else {
                    let display = Self::display_label(token, &lower, platform);
                    ui.add_sized(
                        fixed,
                        egui::Label::new(
                            egui::RichText::new(display)
                                .size(KEY_CAP_FONT_SIZE)
                                .color(text_color),
                        ),
                    );
                }
            });
        } else {
            /* Long keys (Esc, Backspace…) → auto-width, same vertical padding */
            frame.show(ui, |ui| {
                /* WHY: Match the same inner height as fixed-size keys so all caps
                in a row share identical outer height. */
                ui.set_max_height(KEY_CAP_FIXED_H);
                let mut disp = Self::display_label(token, &lower, platform);
                if lower == "tab" {
                    disp = "⇥".to_string();
                } else if lower == "enter" || lower == "return" {
                    disp = "⏎".to_string();
                }
                ui.label(
                    egui::RichText::new(disp)
                        .size(KEY_CAP_FONT_SIZE)
                        .color(text_color),
                );
            });
        }
    }

    fn modifier_icon(lower: &str, platform: ShortcutDisplayPlatform) -> Option<crate::icon::Icon> {
        match lower {
            "primary" | "cmd" | "mac_cmd" | "\u{2318}"
                if platform == ShortcutDisplayPlatform::Macos =>
            {
                Some(crate::icon::Icon::MacCmd)
            }
            "ctrl" | "control" | "\u{2303}" => Some(crate::icon::Icon::MacCtrl),
            "shift" | "\u{21e7}" => Some(crate::icon::Icon::MacShift),
            "alt" | "option" | "opt" | "\u{2325}" => Some(crate::icon::Icon::MacAlt),
            _ => None,
        }
    }

    fn display_label(token: &str, lower: &str, platform: ShortcutDisplayPlatform) -> String {
        if platform == ShortcutDisplayPlatform::NonMacos
            && matches!(lower, "primary" | "cmd" | "mac_cmd" | "\u{2318}")
        {
            return "Ctrl".to_string();
        }
        if token == "\\" || token == "¥" {
            return "¥".to_string();
        }
        token.to_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_macos_primary_modifier_displays_ctrl_text() {
        assert_eq!(
            KeyCapOps::display_label("primary", "primary", ShortcutDisplayPlatform::NonMacos),
            "Ctrl"
        );
    }

    #[test]
    fn non_macos_primary_modifier_does_not_use_windows_icon() {
        assert!(KeyCapOps::modifier_icon("primary", ShortcutDisplayPlatform::NonMacos).is_none());
    }
}
