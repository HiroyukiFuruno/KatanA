use eframe::egui;
use egui_commonmark::CommonMarkCache;

#[path = "markdown_renderer.rs"]
mod markdown_renderer;

pub(crate) struct MarkdownUiOps;

impl MarkdownUiOps {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_markdown(
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        md: &str,
        source_lines: usize,
        md_file_path: &std::path::Path,
        heading_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        block_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        heading_offset: usize,
        global_task_list_idx: &mut usize,
        active_editor_line: Option<usize>,
        hovered_lines: &mut Option<&mut Vec<std::ops::Range<usize>>>,
        global_line_offset: usize,
        search_query: Option<String>,
        search_active_index: Option<usize>,
        is_slideshow: bool,
        is_last_section: bool,
    ) -> Vec<(usize, char)> {
        markdown_renderer::MarkdownRendererOps::render_markdown(
            ui,
            cache,
            md,
            source_lines,
            md_file_path,
            heading_anchors,
            block_anchors,
            heading_offset,
            global_task_list_idx,
            active_editor_line,
            hovered_lines,
            global_line_offset,
            search_query,
            search_active_index,
            is_slideshow,
            is_last_section,
        )
    }
}
