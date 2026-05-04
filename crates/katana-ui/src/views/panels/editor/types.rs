use eframe::egui;
use katana_core::editor::{EditorConfig, EditorWidget};

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

/// Result of a Markdown authoring transform applied to a buffer.
pub struct AuthoringTransform {
    /// The updated buffer after applying the transform.
    pub buffer: String,
    /// Byte offset of the cursor / selection start in the updated buffer.
    pub cursor_start: usize,
    /// Byte offset of the cursor / selection end in the updated buffer.
    pub cursor_end: usize,
}

pub(crate) struct MarkdownEditorWidget {
    config: EditorConfig,
}

impl MarkdownEditorWidget {
    pub(crate) fn new() -> Self {
        Self {
            config: EditorConfig::default(),
        }
    }
}

impl EditorWidget for MarkdownEditorWidget {
    fn config(&self) -> &EditorConfig {
        &self.config
    }

    fn apply_config(&mut self, config: EditorConfig) {
        self.config = config;
    }
}
