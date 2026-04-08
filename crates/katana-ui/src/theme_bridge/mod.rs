pub mod logic;
pub mod types;

pub use types::*;

impl ThemeBridgeOps {
    /* WHY: Basic color wrappers to avoid direct egui dependency in all modules. */
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
