use eframe::egui;
use std::path::Path;

pub struct HtmlRenderer<'a> {
    pub(crate) ui: &'a mut egui::Ui,
    pub(crate) _base_dir: &'a Path,
    pub(crate) text_color: Option<egui::Color32>,
    pub(crate) max_image_width: f32,
    pub(crate) is_strong: bool,
    pub(crate) is_italics: bool,
}

impl HtmlRenderer<'_> {
    pub(crate) fn trace_layout(&self, stage: &str, details: std::fmt::Arguments<'_>) {
        if !crate::preview_pane::HtmlLogicOps::layout_trace_active(self.ui.ctx()) {
            return;
        }

        let next = self.ui.next_widget_position();
        let min = self.ui.min_rect();
        crate::debug_log::DebugLog::write(
            "html_renderer_trace",
            format_args!(
                "stage={stage} frame={} {details} body_height={:.1} cursor_height={:.1} item_spacing_y={:.1} next=({:.1},{:.1}) min=({:.1},{:.1},{:.1},{:.1})",
                self.ui.ctx().cumulative_frame_nr(),
                self.ui.text_style_height(&egui::TextStyle::Body),
                self.ui.cursor().height(),
                self.ui.spacing().item_spacing.y,
                next.x,
                next.y,
                min.min.x,
                min.min.y,
                min.max.x,
                min.max.y,
            ),
        );
    }
}
