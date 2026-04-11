use eframe::egui;

pub struct ThemeBridgeOps;

pub(crate) const IMAGE_VIEWER_OVERLAY_ALPHA: u8 = 255;
pub(crate) const IMAGE_VIEWER_OVERLAY_COLOR: egui::Color32 =
    egui::Color32::from_black_alpha(IMAGE_VIEWER_OVERLAY_ALPHA);

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
