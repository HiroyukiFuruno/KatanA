use eframe::egui;

const KEY_CAP_SEP: f32 = 3.0;
const KEY_CAP_FONT_SIZE: f32 = 11.0;

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
        if normalized.is_empty() {
            return ui.label("");
        }
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
                        super::key_cap::KeyCapOps::draw_key_cap(ui, part.trim());
                    }
                }
            } else {
                for (i, combo) in combos.into_iter().enumerate() {
                    if i > 0 {
                        ui.label(egui::RichText::new("/").weak().size(KEY_CAP_FONT_SIZE));
                    }
                    for part in combo.split('+') {
                        super::key_cap::KeyCapOps::draw_key_cap(ui, part.trim());
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
        s.split(", ")
            .map(Self::normalize_combo)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn normalize_combo(combo: &str) -> String {
        let mut seen_primary = false;
        combo
            .split('+')
            .filter_map(|p| {
                let t = p.trim();
                if matches!(t.to_lowercase().as_str(), "primary" | "cmd" | "mac_cmd") {
                    if seen_primary {
                        return None;
                    }
                    seen_primary = true;
                    Some("primary")
                } else if t == "¥" || t == "|" {
                    /* WHY: ¥ (U+00A5) and | (Shift+¥) are both the JIS Backslash key.
                    Normalize to "\\" so legacy saved shortcuts are always canonical. */
                    Some("\\")
                } else {
                    Some(t)
                }
            })
            .collect::<Vec<_>>()
            .join("+")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_combo_maps_yen_to_backslash() {
        /* WHY: ¥ and \\ are the same physical JIS key. After normalization the token must
        be canonical "\\" so that display and shortcut matching are consistent. */
        assert_eq!(
            ShortcutWidget::normalize_shortcut("primary+¥"),
            "primary+\\"
        );
    }

    #[test]
    fn normalize_combo_leaves_backslash_unchanged() {
        assert_eq!(
            ShortcutWidget::normalize_shortcut("primary+\\"),
            "primary+\\"
        );
    }

    #[test]
    fn normalize_combo_shift_yen_maps_to_shift_backslash() {
        assert_eq!(
            ShortcutWidget::normalize_shortcut("primary+Shift+¥"),
            "primary+Shift+\\"
        );
    }
}
