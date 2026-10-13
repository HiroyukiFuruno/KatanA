use eframe::egui;

pub struct Modal<'a> {
    pub(crate) id: &'a str,
    pub(crate) title: &'a str,
    pub(crate) progress: Option<f32>,
    pub(crate) show_pct: bool,
    pub(crate) bar_width: f32,
    pub(crate) width: Option<f32>,
    pub(crate) fixed_size: Option<egui::Vec2>,
    pub(crate) frame: Option<egui::Frame>,
    pub(crate) window_controls: Option<ModalWindowControls<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalWindowButton {
    Close,
    Fullscreen,
}

#[derive(Clone, Copy)]
pub struct ModalWindowControls<'a> {
    pub is_fullscreen: bool,
    pub show_fullscreen: bool,
    pub close_tooltip: &'a str,
    pub enter_fullscreen_tooltip: &'a str,
    pub exit_fullscreen_tooltip: &'a str,
}
