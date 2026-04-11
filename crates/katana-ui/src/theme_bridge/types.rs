use eframe::egui;

pub struct ThemeBridgeOps;

pub const WHITE: egui::Color32 = egui::Color32::WHITE;
pub const BLACK: egui::Color32 = egui::Color32::BLACK;
pub const TRANSPARENT: egui::Color32 = egui::Color32::TRANSPARENT;
pub const INVISIBLE: egui::Color32 = egui::Color32::from_black_alpha(1);

pub(crate) const LIGHT_MODE_ICON_BG: u8 = 245;

impl ThemeBridgeOps {
    pub fn light_mode_icon_bg() -> egui::Color32 {
        egui::Color32::from_gray(LIGHT_MODE_ICON_BG)
    }
}
