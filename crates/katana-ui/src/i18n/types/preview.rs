use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewMessages {
    pub preview_title: String,
    pub refresh_diagrams: String,
    pub rendering: String,
    pub no_preview: String,
    pub slideshow_settings: String,
    pub highlight_hover: String,
    pub show_diagram_controls: String,
    pub diagram_controller: DiagramControllerMessages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramControllerMessages {
    pub pan_up: String,
    pub pan_down: String,
    pub pan_left: String,
    pub pan_right: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub reset: String,
    pub fullscreen: String,
    pub close: String,
    pub trackpad_help: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantUmlMessages {
    pub downloading_plantuml: String,
    pub plantuml_installed: String,
    pub download_error: String,
}
