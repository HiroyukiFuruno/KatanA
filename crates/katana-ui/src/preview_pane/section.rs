use crate::preview_pane::{DownloadRequest, RenderedSection, SectionLifecycle, ViewerState};
use eframe::egui::{self};
use egui_commonmark::CommonMarkCache;

pub use super::types::SectionLogicOps;

impl SectionLogicOps {
    pub(crate) fn mark_drawn_and_anchor(
        rect: egui::Rect,
        i: usize,
        lines_in_section: usize,
        global_line_offset: usize,
        section_lifecycle: &mut Option<&mut Vec<SectionLifecycle>>,
        block_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    ) {
        if let Some(lifecycle) = section_lifecycle.as_mut().filter(|l| i < l.len()) {
            lifecycle[i].is_drawn = true;
        }
        if let Some(anchors) = block_anchors.as_mut() {
            anchors.push((
                global_line_offset..global_line_offset + lines_in_section,
                rect,
            ));
        }
    }

    #[allow(unused_mut, clippy::too_many_arguments)]
    pub fn show_section(
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        section: &RenderedSection,
        id: usize,
        md_file_path: &std::path::Path,
        scroll_to_heading_index: Option<usize>,
        heading_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        block_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        heading_offset: usize,
        global_task_list_idx: &mut usize,
        active_editor_line: Option<usize>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
        global_line_offset: usize,
        search_query: Option<String>,
        search_scroll_pending: bool,
        is_slideshow: bool,
    ) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        super::section_show::show_section(
            ui,
            cache,
            section,
            id,
            md_file_path,
            scroll_to_heading_index,
            heading_anchors,
            block_anchors,
            heading_offset,
            global_task_list_idx,
            active_editor_line,
            hovered_lines,
            global_line_offset,
            search_query,
            search_scroll_pending,
            is_slideshow,
        )
    }

    #[allow(unused_mut, clippy::too_many_arguments)]
    pub fn render_sections(
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        sections: &[RenderedSection],
        md_file_path: &std::path::Path,
        scroll_to_heading_index: Option<usize>,
        mut heading_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        mut block_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        mut viewer_states: Option<&mut Vec<ViewerState>>,
        mut section_lifecycle: Option<&mut Vec<SectionLifecycle>>,
        mut fullscreen_request: Option<&mut Option<usize>>,
        active_editor_line: Option<usize>,
        mut hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
        search_query: Option<String>,
        search_scroll_pending: bool,
        is_slideshow: bool,
    ) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
        let mut request: Option<DownloadRequest> = None;
        let mut actions = Vec::new();
        let mut current_heading_offset = 0;
        let mut global_task_list_idx = 0;
        let mut global_line_offset = 0;

        for (i, section) in sections.iter().enumerate() {
            ui.push_id(format!("section_{i}"), |ui| {
                let mut offset = 0;
                let lines_in_section = if let RenderedSection::Markdown(md) = section {
                    offset = current_heading_offset;
                    current_heading_offset +=
                        katana_core::markdown::outline::MarkdownOutlineOps::extract_outline(md)
                            .len();
                    md.chars().filter(|c| *c == '\n').count()
                } else {
                    match section {
                        RenderedSection::Image { source_lines, .. } => *source_lines,
                        RenderedSection::LocalImage { source_lines, .. } => *source_lines,
                        RenderedSection::Error { source_lines, .. } => *source_lines,
                        RenderedSection::NotInstalled { source_lines, .. } => *source_lines,
                        RenderedSection::Pending { source_lines, .. } => *source_lines,
                        RenderedSection::CommandNotFound { source_lines, .. } => *source_lines,
                        _ => 0,
                    }
                };
                match section {
                    RenderedSection::Image { svg_data, alt, .. } => {
                        crate::preview_pane::types::SectionImageOps::handle_image_section(
                            ui,
                            svg_data,
                            alt,
                            i,
                            lines_in_section,
                            global_line_offset,
                            active_editor_line,
                            viewer_states.as_deref_mut(),
                            fullscreen_request.as_deref_mut(),
                            &mut section_lifecycle,
                            &mut block_anchors,
                            hovered_lines.as_deref_mut(),
                            is_slideshow,
                        );
                    }
                    RenderedSection::LocalImage { path, alt, .. } => {
                        crate::preview_pane::types::SectionImageOps::handle_local_image_section(
                            ui,
                            path,
                            alt,
                            i,
                            lines_in_section,
                            global_line_offset,
                            active_editor_line,
                            viewer_states.as_deref_mut(),
                            fullscreen_request.as_deref_mut(),
                            &mut section_lifecycle,
                            &mut block_anchors,
                            hovered_lines.as_deref_mut(),
                            is_slideshow,
                        );
                    }
                    _ => {
                        let (req, mut event_actions) = Self::show_section(
                            ui,
                            cache,
                            section,
                            i,
                            md_file_path,
                            scroll_to_heading_index,
                            heading_anchors.as_deref_mut(),
                            block_anchors.as_deref_mut(),
                            offset,
                            &mut global_task_list_idx,
                            active_editor_line,
                            hovered_lines.as_deref_mut(),
                            global_line_offset,
                            search_query.clone(),
                            search_scroll_pending,
                            is_slideshow,
                        );
                        if let Some(lifecycle) = section_lifecycle.as_mut()
                            && i < lifecycle.len()
                        {
                            lifecycle[i].is_drawn = true; // Markdown and non-image error sections render directly
                        }
                        if let Some(r) = req {
                            request = Some(r);
                        }
                        actions.append(&mut event_actions);
                    }
                }
                global_line_offset += lines_in_section;
            });
        }
        if sections.is_empty() {
            ui.label(
                egui::RichText::new(crate::i18n::I18nOps::get().preview.no_preview.clone()).weak(),
            );
        }
        (request, actions)
    }
}
