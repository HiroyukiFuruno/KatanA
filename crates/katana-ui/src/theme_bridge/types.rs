use eframe::egui;

pub struct ThemeBridgeOps;

pub(crate) const IMAGE_VIEWER_OVERLAY_ALPHA: u8 = 180;
pub(crate) const IMAGE_VIEWER_OVERLAY_COLOR: egui::Color32 =
    egui::Color32::from_black_alpha(IMAGE_VIEWER_OVERLAY_ALPHA);

pub const WHITE: egui::Color32 = egui::Color32::WHITE;
pub const BLACK: egui::Color32 = egui::Color32::BLACK;
pub const TRANSPARENT: egui::Color32 = egui::Color32::TRANSPARENT;
