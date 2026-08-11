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
    pub document_controller: DocumentControllerMessages,
    #[serde(default = "default_missing_image_text")]
    pub missing_image: String,
    #[serde(default = "default_remote_image_text")]
    pub remote_image: String,
    pub diagram_controller: DiagramControllerMessages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentControllerMessages {
    pub previous: String,
    pub next: String,
    pub fit_page: String,
    pub fit_width: String,
    pub copy_active_cell: String,
    pub error_details: String,
    pub rendering_notes: String,
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
