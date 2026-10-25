/* WHY: Refactored markdown rendering entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

pub mod anchors;
mod markdown_ui;
pub mod tasks;
pub mod theme;

pub struct SectionMarkdownOps;

impl SectionMarkdownOps {
    #[allow(clippy::too_many_arguments)]
    pub fn render_markdown(
        ui: &mut eframe::egui::Ui,
        cache: &mut egui_commonmark::CommonMarkCache,
        md: &str,
        source_lines: usize,
        md_file_path: &std::path::Path,
        heading_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, eframe::egui::Rect)>>,
        block_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, eframe::egui::Rect)>>,
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
        markdown_ui::MarkdownUiOps::render_markdown(
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
