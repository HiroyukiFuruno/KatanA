use crate::preview_pane::*;
use crate::preview_pane::{DownloadRequest, RenderedSection};
use eframe::egui::{self};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::color_preset::DiagramColorPreset;

#[allow(unused_mut, clippy::too_many_arguments)]
pub(super) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
    md_file_path: &std::path::Path,
    scroll_to_heading_index: Option<usize>,
    mut heading_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    mut block_anchors: Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    heading_offset: usize,
    global_task_list_idx: &mut usize,
    active_editor_line: Option<usize>,
    mut hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
    global_line_offset: usize,
    search_query: Option<String>,
    search_scroll_pending: bool,
    is_slideshow: bool,
) -> (Option<DownloadRequest>, Vec<(usize, char)>) {
    let mut actions = Vec::new();
    match section {
        RenderedSection::Markdown(md) => {
            crate::preview_pane::PreviewPaneUtilsOps::with_preview_text_style(ui, |ui| {
                let preset = if ui.visuals().dark_mode {
                    DiagramColorPreset::dark()
                } else {
                    DiagramColorPreset::light()
                };
                let theme_colors = ui.ctx().data(|d| {
                    d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                        "katana_theme_colors",
                    ))
                });
                let text_color = theme_colors
                    .as_ref()
                    .map(|tc| crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.text));
                let hover_bg_color = theme_colors.as_ref().map(|tc| {
                    crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                        tc.preview.hover_line_background,
                    )
                });
                let border_color = theme_colors.as_ref().map(|tc| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.border)
                });
                let selection_color = theme_colors.as_ref().map(|tc| {
                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.selection)
                });

                let md_path_owned = md_file_path.to_path_buf();
                let binding = move |ui: &mut egui::Ui, html: &str| {
                    crate::preview_pane::HtmlLogicOps::render_html_block(
                        ui,
                        html,
                        text_color,
                        &md_path_owned,
                    );
                };
                let math_binding = |ui: &mut egui::Ui, tex: &str, is_inline: bool| {
                    MathLogicOps::render_math(ui, tex, is_inline);
                };

                let mut viewer = CommonMarkViewer::new()
                    .syntax_theme_dark(preset.syntax_theme_dark)
                    .syntax_theme_light(preset.syntax_theme_light)
                    .search_query(search_query.clone())
                    .search_scroll_pending(search_scroll_pending)
                    .heading_offset(heading_offset)
                    .render_html_fn(Some(&binding))
                    .render_math_fn(Some(&math_binding))
                    .hover_bg_color(hover_bg_color)
                    .show_code_copy_button(!is_slideshow)
                    .custom_task_box_fn(Some(&crate::widgets::MarkdownHooksOps::katana_task_box))
                    .custom_task_context_menu_fn(Some(
                        &crate::widgets::MarkdownHooksOps::katana_task_context_menu,
                    ))
                    .custom_emoji_fn(Some(
                        &katana_core::emoji::EmojiRasterOps::render_apple_color_emoji_png,
                    ));

                if let Some(idx) = scroll_to_heading_index {
                    viewer = viewer.scroll_to_heading_index(idx);
                }

                let previous_anchor_count = heading_anchors.as_ref().map(|a| a.len()).unwrap_or(0);
                if let Some(anchors) = heading_anchors.as_mut() {
                    viewer = viewer.heading_anchors(anchors);
                }

                let mut local_block_anchors = Vec::new();
                if block_anchors.is_some() {
                    viewer = viewer.block_anchors(&mut local_block_anchors);
                }

                /* WHY: Compute the active char range once so it can be shared with the */
                /* WHY: list-item highlight callback and the viewer builder. */
                let mut computed_active_range: Option<std::ops::Range<usize>> = None;
                if let Some(global_line) = active_editor_line {
                    let lines_in_md = md.chars().filter(|c| *c == '\n').count();
                    if global_line >= global_line_offset
                        && global_line <= global_line_offset + lines_in_md
                    {
                        let local_line = global_line - global_line_offset;
                        computed_active_range = byte_range_for_line(md, local_line);
                    }
                }

                if let Some(ref range) = computed_active_range {
                    viewer = viewer.active_char_range(range.clone());
                }

                /* WHY: Both colors resolved from ThemeColors; fallback to hover_line_background */
                /* WHY: since PreviewColors has no dedicated active_line_background. */
                let resolved_hover = hover_bg_color.unwrap_or(crate::theme_bridge::TRANSPARENT);
                let list_highlight_fn =
                    crate::widgets::MarkdownHooksOps::katana_list_item_highlight(
                        computed_active_range.clone(),
                        resolved_hover,
                        resolved_hover,
                    );
                viewer = viewer.custom_list_item_highlight_fn(Some(&list_highlight_fn));

                let slideshow_hover = ui.ctx().data(|d| {
                    d.get_temp(egui::Id::new("katana_slideshow_hover_highlight"))
                        .unwrap_or(false)
                });
                let mut local_hovered_spans = Vec::new();
                if hovered_lines.is_some() || (is_slideshow && slideshow_hover) {
                    viewer = viewer.hovered_spans(&mut local_hovered_spans);
                }
                viewer = viewer.search_query(search_query.clone());

                let (_, newly_captured) = ui
                    .scope(|ui| {
                        if let Some(color) = text_color {
                            ui.visuals_mut().override_text_color = Some(color);
                        }
                        if let Some(border) = border_color {
                            ui.visuals_mut().widgets.noninteractive.bg_stroke.color = border;
                        }
                        const TABLE_STRIPE_ALPHA: f32 = 0.1;
                        if let Some(sel) = selection_color {
                            ui.visuals_mut().selection.bg_fill = sel;
                            ui.visuals_mut().faint_bg_color =
                                sel.gamma_multiply(TABLE_STRIPE_ALPHA);
                        }
                        viewer.show_with_events(ui, cache, md)
                    })
                    .inner;

                if let Some(anchors) = heading_anchors {
                    for anchor in &mut anchors[previous_anchor_count..] {
                        let local_span = &anchor.0;
                        let start_line = global_line_offset
                            + md[..local_span.start]
                                .chars()
                                .filter(|c| *c == '\n')
                                .count();
                        let end_line = global_line_offset
                            + md[..local_span.end].chars().filter(|c| *c == '\n').count();
                        anchor.0 = start_line..end_line;
                    }
                }
                if let Some(anchors) = block_anchors {
                    for (local_span, rect) in local_block_anchors {
                        let start_line = global_line_offset
                            + md[..local_span.start]
                                .chars()
                                .filter(|c| *c == '\n')
                                .count();
                        let end_line = global_line_offset
                            + md[..local_span.end].chars().filter(|c| *c == '\n').count();
                        anchors.push((start_line..end_line, rect));
                    }
                }
                if let Some(hovered) = hovered_lines {
                    for local_span in local_hovered_spans {
                        let start_line = global_line_offset
                            + md[..local_span.start]
                                .chars()
                                .filter(|c| *c == '\n')
                                .count();
                        /* WHY: Use saturating_sub(1) to exclude trailing newline */
                        /* WHY: that pulldown_cmark includes in source spans. */
                        let end_pos = local_span.end.saturating_sub(1).max(local_span.start);
                        let end_line = global_line_offset
                            + md[..end_pos].chars().filter(|c| *c == '\n').count();
                        hovered.push(start_line..end_line);
                    }
                }
                let spans = egui_commonmark::extract_task_list_spans(md);
                for action in newly_captured {
                    if let Some(local_idx) = spans.iter().position(|s| s == &action.span) {
                        actions.push((*global_task_list_idx + local_idx, action.new_state));
                    }
                }
                *global_task_list_idx += spans.len();
            });
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
                .color(warning_color(ui))
                .small(),
            );
            if let Some(anchors) = block_anchors {
                anchors.push((
                    global_line_offset..global_line_offset + source_lines,
                    res.rect,
                ));
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
            let res = ui.label(egui::RichText::new(msg).color(warning_color(ui)).small());
            if let Some(anchors) = block_anchors {
                anchors.push((
                    global_line_offset..global_line_offset + source_lines,
                    res.rect,
                ));
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
            (None, vec![])
        }
    }
}

fn byte_range_for_line(md: &str, local_line: usize) -> Option<std::ops::Range<usize>> {
    let mut current_line = 0;
    let mut start_byte = None;
    let mut end_byte = None;
    for (i, c) in md.char_indices() {
        if current_line == local_line && start_byte.is_none() {
            start_byte = Some(i);
        }
        if current_line == local_line + 1 {
            end_byte = Some(i);
            break;
        }
        if c == '\n' {
            current_line += 1;
        }
    }
    if current_line == local_line && start_byte.is_none() {
        start_byte = Some(0);
    }
    start_byte.map(|s| s..end_byte.unwrap_or(md.len()))
}

fn warning_color(ui: &egui::Ui) -> egui::Color32 {
    ui.ctx()
        .data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        })
        .map_or(crate::theme_bridge::WHITE, |tc| {
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.warning_text)
        })
}
