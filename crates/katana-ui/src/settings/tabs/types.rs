use katana_platform::theme::{Rgb, Rgba, ThemeColors};

pub struct BehaviorTabOps;
pub struct FontTabOps;
pub struct IconsTabOps;
pub struct LayoutTabOps;
pub struct ThemeTabOps;
pub struct UpdatesTabOps;
pub struct WorkspaceTabOps;
pub struct ShortcutsTabOps;
pub struct LinterTabOps;
pub struct AiTabOps;

pub(crate) enum ColorPropType {
    Rgb(fn(&ThemeColors) -> Rgb, fn(&mut ThemeColors, Rgb)),
    Rgba(fn(&ThemeColors) -> Rgba, fn(&mut ThemeColors, Rgba)),
}

use crate::settings::*;
use eframe::egui;

impl ColorPropType {
    pub(crate) fn render_row(
        &self,
        ui: &mut egui::Ui,
        new_colors: &mut ThemeColors,
        label: &str,
    ) -> bool {
        let mut changed = false;
        match self {
            ColorPropType::Rgb(get, apply) => {
                let mut color =
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(get(new_colors));
                if crate::widgets::LabeledColorPicker::new(label)
                    .label_width(COLOR_GRID_LABEL_WIDTH)
                    .spacing(SECTION_SPACING)
                    .show_rgb(ui, &mut color)
                    .changed()
                {
                    apply(
                        new_colors,
                        Rgb {
                            r: color.r(),
                            g: color.g(),
                            b: color.b(),
                        },
                    );
                    changed = true;
                }
            }
            ColorPropType::Rgba(get, apply) => {
                let mut color =
                    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(get(new_colors));
                if crate::widgets::LabeledColorPicker::new(label)
                    .label_width(COLOR_GRID_LABEL_WIDTH)
                    .spacing(SECTION_SPACING)
                    .show_rgba(ui, &mut color)
                    .changed()
                {
                    let [r, g, b, a] = color.to_srgba_unmultiplied();
                    apply(new_colors, Rgba { r, g, b, a });
                    changed = true;
                }
            }
        }
        changed
    }
}
