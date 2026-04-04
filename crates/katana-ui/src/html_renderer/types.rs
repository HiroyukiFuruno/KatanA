use eframe::egui;
use std::path::Path;

pub struct HtmlRenderer<'a> {
    pub(crate) ui: &'a mut egui::Ui,
    pub(crate) _base_dir: &'a Path,
    pub(crate) text_color: Option<egui::Color32>,
    pub(crate) max_image_width: f32,
}
