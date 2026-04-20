/* WHY: Refactored section entry point to maintain a clean top-level structure and manage complexity via sub-modules. */

use crate::preview_pane::{DownloadRequest, RenderedSection};
use eframe::egui;
use egui_commonmark::CommonMarkCache;

mod markdown;
mod render_utils;

#[allow(unused_mut, clippy::too_many_arguments)]
pub(super) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
    md_file_path: &std::path::Path,
    _scroll_to_heading_index: Option<usize>,
    mut heading_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    mut block_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    heading_offset: usize,
    global_task_list_idx: &mut usize,
    active_editor_line: Option<usize>,
    mut hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    global_line_offset: usize,
    search_query: Option<String>,
    search_active_index: Option<usize>,
    is_slideshow: bool,
    is_last_section: bool,
) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
    match section {
        RenderedSection::Markdown(md, source_lines) => {
            let actions = markdown::SectionMarkdownOps::render_markdown(
                ui,
                cache,
                md,
                *source_lines,
                md_file_path,
                &mut heading_anchors,
                &mut block_anchors,
                heading_offset,
                global_task_list_idx,
                active_editor_line,
                &mut hovered_lines,
                global_line_offset,
                search_query,
                search_active_index,
                is_slideshow,
                is_last_section,
            );
            (None, actions)
        }
        RenderedSection::Image { svg_data, alt, .. } => {
            crate::preview_pane::ImageLogicOps::show_rasterized(
                ui,
                svg_data,
                alt,
                id,
                None,
                None,
                |_, _, _| {},
            );
            (None, vec![])
        }
        RenderedSection::LocalImage { path, alt, .. } => {
            crate::preview_pane::ImageLogicOps::show_local_image(
                ui,
                path,
                alt,
                id,
                None,
                None,
                |_, _, _| {},
            );
            (None, vec![])
        }
        RenderedSection::Error {
            kind,
            message,
            source_lines,
            ..
        } => {
            let res = ui.label(
                egui::RichText::new(crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().error.render_error,
                    &[("kind", kind.as_str()), ("message", message.as_str())],
                ))
                .color(render_utils::SectionRenderUtilsOps::warning_color(ui))
                .small(),
            );
            if let Some(anchors) = block_anchors {
                anchors.push((
                    global_line_offset..global_line_offset + source_lines,
                    res.rect,
                ));
            }
            if let Some(hovered) = hovered_lines
                && res.hovered()
            {
                hovered.push(global_line_offset..global_line_offset + source_lines);
            }
            (None, vec![])
        }
        RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            source_lines,
            ..
        } => {
            let msg = crate::i18n::I18nOps::get()
                .error
                .missing_dependency
                .clone()
                .replace("{tool_name}", tool_name)
                .replace("{install_hint}", install_hint);
            let res = ui.label(
                egui::RichText::new(msg)
                    .color(render_utils::SectionRenderUtilsOps::warning_color(ui))
                    .small(),
            );
            if let Some(anchors) = block_anchors {
                anchors.push((
                    global_line_offset..global_line_offset + source_lines,
                    res.rect,
                ));
            }
            if let Some(hovered) = hovered_lines
                && res.hovered()
            {
                hovered.push(global_line_offset..global_line_offset + source_lines);
            }
            (None, vec![])
        }
        RenderedSection::NotInstalled {
            kind,
            download_url,
            install_path,
            source_lines,
            ..
        } => {
            let (rect, req) =
                crate::preview_pane::image_fallback::ImageFallbackOps::show_not_installed(
                    ui,
                    kind,
                    download_url,
                    install_path,
                );
            if let Some(anchors) = block_anchors {
                anchors.push((global_line_offset..global_line_offset + source_lines, rect));
            }
            if let Some(hovered) = hovered_lines
                && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                && rect.contains(pos)
            {
                hovered.push(global_line_offset..global_line_offset + source_lines);
            }
            (req, vec![])
        }
        RenderedSection::Pending {
            kind, source_lines, ..
        } => {
            let res = crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.spinner();
                    ui.label(
                        egui::RichText::new(crate::i18n::I18nOps::tf(
                            &crate::i18n::I18nOps::get().preview.rendering,
                            &[("kind", kind.as_str())],
                        ))
                        .weak(),
                    );
                })
                .show(ui);
            if let Some(anchors) = block_anchors {
                anchors.push((
                    global_line_offset..global_line_offset + source_lines,
                    res.rect,
                ));
            }
            if let Some(hovered) = hovered_lines
                && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                && res.rect.contains(pos)
            {
                hovered.push(global_line_offset..global_line_offset + source_lines);
            }
            (None, vec![])
        }
    }
}
