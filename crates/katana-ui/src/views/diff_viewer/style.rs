use eframe::egui;
use katana_platform::theme::{Rgb, Rgba, ThemeColors};

const CHANGE_BACKGROUND_ALPHA: u8 = 42;

#[derive(Debug, Clone, Copy)]
pub(super) enum DiffTone {
    Normal,
    Removed,
    Added,
    Collapsed,
}

pub(super) struct DiffViewerPalette {
    pub(super) text: egui::Color32,
    pub(super) secondary_text: egui::Color32,
    pub(super) removed_text: egui::Color32,
    pub(super) added_text: egui::Color32,
    pub(super) normal_background: egui::Color32,
    pub(super) removed_background: egui::Color32,
    pub(super) added_background: egui::Color32,
    pub(super) collapsed_background: egui::Color32,
    pub(super) gutter_background: egui::Color32,
}

impl DiffViewerPalette {
    pub(super) fn from_ui(ui: &egui::Ui) -> Self {
        if let Some(colors) = theme_colors(ui) {
            return Self::from_theme(ui, &colors);
        }

        Self {
            text: ui.visuals().text_color(),
            secondary_text: ui.visuals().widgets.inactive.fg_stroke.color,
            removed_text: ui.visuals().error_fg_color,
            added_text: ui.visuals().hyperlink_color,
            normal_background: ui.visuals().code_bg_color,
            removed_background: ui.visuals().faint_bg_color,
            added_background: ui.visuals().selection.bg_fill,
            collapsed_background: ui.visuals().widgets.inactive.bg_fill,
            gutter_background: ui.visuals().faint_bg_color,
        }
    }

    fn from_theme(ui: &egui::Ui, colors: &ThemeColors) -> Self {
        let removed_text =
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.error_text);
        let added_text =
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.success_text);
        Self {
            text: crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(colors.system.text),
            secondary_text: crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                colors.system.text_secondary,
            ),
            removed_text,
            added_text,
            normal_background: crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                colors.code.background,
            ),
            removed_background: change_background(colors.system.error_text),
            added_background: change_background(colors.system.success_text),
            collapsed_background: ui.visuals().widgets.inactive.bg_fill,
            gutter_background: ui.visuals().faint_bg_color,
        }
    }

    pub(super) fn text_for(&self, tone: DiffTone) -> egui::Color32 {
        match tone {
            DiffTone::Normal | DiffTone::Collapsed => self.text,
            DiffTone::Removed => self.removed_text,
            DiffTone::Added => self.added_text,
        }
    }

    pub(super) fn background_for(&self, tone: DiffTone) -> egui::Color32 {
        match tone {
            DiffTone::Normal => self.normal_background,
            DiffTone::Removed => self.removed_background,
            DiffTone::Added => self.added_background,
            DiffTone::Collapsed => self.collapsed_background,
        }
    }
}

fn change_background(color: Rgb) -> egui::Color32 {
    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(Rgba {
        r: color.r,
        g: color.g,
        b: color.b,
        a: CHANGE_BACKGROUND_ALPHA,
    })
}

fn theme_colors(ui: &egui::Ui) -> Option<ThemeColors> {
    ui.data(|data| data.get_temp::<ThemeColors>(egui::Id::new("katana_theme_colors")))
}
