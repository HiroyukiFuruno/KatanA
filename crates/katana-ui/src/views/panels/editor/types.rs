use eframe::egui;

/// Editor color tuple: (code_bg, code_text, code_selection, current_line_bg, hover_line_bg, ln_text, ln_active_text).
pub type EditorColors = (
    egui::Color32,
    egui::Color32,
    Option<egui::Color32>,
    Option<egui::Color32>,
    Option<egui::Color32>,
    Option<egui::Color32>,
    Option<egui::Color32>,
);

pub struct EditorLogicOps;
