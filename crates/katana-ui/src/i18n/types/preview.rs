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
    pub toggle_slideshow: String,
    #[serde(default = "default_missing_image_text")]
    pub missing_image: String,
    #[serde(default = "default_remote_image_text")]
    pub remote_image: String,
    pub diagram_controller: DiagramControllerMessages,
}

fn default_missing_image_text() -> String {
    "Missing Local Image".to_string()
}
fn default_remote_image_text() -> String {
    "Remote Image URL".to_string()
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
    #[serde(default = "default_downloading_tool")]
    pub downloading_tool: String,
    #[serde(default = "default_tool_installed")]
    pub tool_installed: String,
}

fn default_downloading_tool() -> String {
    "Downloading {tool}...".to_string()
}

fn default_tool_installed() -> String {
    "{tool} installed. Refreshing preview...".to_string()
}
