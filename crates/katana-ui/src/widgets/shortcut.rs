use eframe::egui;

/// Key cap rendering constants
const KEY_CAP_ROUNDING: f32 = 4.0;
/// Shared horizontal padding for ALL key caps.
const KEY_CAP_MARGIN_X: i8 = 4;
/// Shared vertical padding for ALL key caps.
const KEY_CAP_MARGIN_Y: i8 = 2;
/// Gap between adjacent key cap badges.
const KEY_CAP_SEP: f32 = 3.0;
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

pub struct ShortcutWidget<'a> {
    shortcut_str: &'a str,
}

impl<'a> ShortcutWidget<'a> {
    pub fn new(shortcut_str: &'a str) -> Self {
        Self { shortcut_str }
    }

    /* WHY: Renders key-cap badges for a full shortcut string (may have multiple combos).
    Adapts to the parent's layout direction natively. If the parent is RightToLeft,
    it adds the elements in reverse order so they appear visually LeftToRight,
    without breaking the container's shrink-wrap sizing. If parent is TopDown (e.g. in a vertical modal),
    it forces a horizontal layout. */
    pub fn ui(&self, ui: &mut egui::Ui) -> egui::Response {
        let normalized = Self::normalize_shortcut(self.shortcut_str);
        let parent_dir = ui.layout().main_dir();

        let render_keys = |ui: &mut egui::Ui, is_rtl: bool| {
            ui.spacing_mut().item_spacing.x = KEY_CAP_SEP;
            let combos: Vec<_> = normalized.split(", ").collect();

            if is_rtl {
                for (i, combo) in combos.into_iter().rev().enumerate() {
                    if i > 0 {
                        ui.label(egui::RichText::new("/").weak().size(KEY_CAP_FONT_SIZE));
                    }
                    let parts: Vec<_> = combo.split('+').collect();
                    for part in parts.into_iter().rev() {
                        Self::draw_key_cap(ui, part.trim());
                    }
                }
            } else {
                for (i, combo) in combos.into_iter().enumerate() {
                    if i > 0 {
                        ui.label(egui::RichText::new("/").weak().size(KEY_CAP_FONT_SIZE));
                    }
                    for part in combo.split('+') {
                        Self::draw_key_cap(ui, part.trim());
                    }
                }
            }
        };

        if parent_dir == egui::Direction::LeftToRight || parent_dir == egui::Direction::RightToLeft
        {
            ui.scope(|ui| render_keys(ui, parent_dir == egui::Direction::RightToLeft))
                .response
        } else {
            ui.allocate_ui_with_layout(
                egui::vec2(0.0, 0.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| render_keys(ui, false),
            )
            .response
        }
    }

    /* WHY: Normalizes stored shortcut strings before display.
    Handles legacy "primary+mac_cmd+X" → "primary+X" (double-modifier bug). */
    fn normalize_shortcut(s: &str) -> String {
        s.split(", ").map(Self::normalize_combo).collect::<Vec<_>>().join(", ")
    }

    fn normalize_combo(combo: &str) -> String {
        let mut seen_primary = false;
        combo.split('+').filter_map(|p| {
            let t = p.trim();
            if matches!(t.to_lowercase().as_str(), "primary" | "cmd" | "mac_cmd") {
                if seen_primary { return None; }
                seen_primary = true;
                Some("primary")
            } else {
                Some(t)
            }
        }).collect::<Vec<_>>().join("+")
    }

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

        let icon_opt = match lower.as_str() {
            "primary" | "cmd" | "mac_cmd" => {
                #[cfg(target_os = "macos")]
                {
                    Some(crate::icon::Icon::MacCmd)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Some(crate::icon::Icon::Windows)
                }
            }
            "ctrl" | "control" => Some(crate::icon::Icon::MacCtrl),
            "shift" => Some(crate::icon::Icon::MacShift),
            "alt" | "option" | "opt" => Some(crate::icon::Icon::MacAlt),
            _ => None,
        };

        let mut stroke = ui.visuals().widgets.inactive.bg_stroke;
        /* WHY: The theme sets bg_stroke color to INVISIBLE (alpha≈0) while keeping width non-zero.
        Check both width AND alpha to avoid drawing an invisible border. */
        if stroke.width == 0.0 || stroke.color.a() < MIN_VISIBLE_ALPHA {
            stroke = egui::Stroke::new(
                STROKE_WIDTH,
                text_color.linear_multiply(STROKE_FALLBACK_OPACITY),
            );
        }

        let mut frame = egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(KEY_CAP_MARGIN_X, KEY_CAP_MARGIN_Y))
            .rounding(KEY_CAP_ROUNDING)
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
                    ui.add_sized(
                        fixed,
                        egui::Label::new(
                            egui::RichText::new(token.to_uppercase())
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
                let mut disp = token.to_uppercase();
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
}
