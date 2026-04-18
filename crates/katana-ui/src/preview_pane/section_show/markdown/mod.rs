/* WHY: Refactored markdown rendering entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

use super::render_utils::SectionRenderUtilsOps;
use crate::preview_pane::*;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::color_preset::DiagramColorPreset;

pub mod anchors;
pub mod tasks;
pub mod theme;

pub struct SectionMarkdownOps;

impl SectionMarkdownOps {
    #[allow(clippy::too_many_arguments)]
    pub fn render_markdown(
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
        let mut actions = Vec::new();
        crate::preview_pane::PreviewPaneUtilsOps::with_preview_text_style(ui, |ui| {
            let preset = if ui.visuals().dark_mode {
                DiagramColorPreset::dark()
            } else {
                DiagramColorPreset::light()
            };

            let theme_colors = theme::MarkdownThemeOps::extract_theme_colors(ui);

            let md_path_owned = md_file_path.to_path_buf();
            let binding = move |ui: &mut egui::Ui, html: &str| {
                crate::preview_pane::HtmlLogicOps::render_html_block(
                    ui,
                    html,
                    theme_colors.text,
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
                .heading_offset(heading_offset)
                .render_html_fn(Some(&binding))
                .render_math_fn(Some(&math_binding))
                .render_footnotes(is_last_section)
                .hover_bg_color(theme_colors.hover_bg)
                .show_code_copy_button(!is_slideshow)
                .custom_task_box_fn(Some(&crate::widgets::MarkdownHooksOps::katana_task_box))
                .custom_task_context_menu_fn(Some(
                    &crate::widgets::MarkdownHooksOps::katana_task_context_menu,
                ))
                .custom_emoji_fn(Some(
                    &katana_core::emoji::EmojiRasterOps::render_apple_color_emoji_png,
                ))
                .render_table_fn(Some(
                    &crate::preview_pane::extension_table::KatanaTableRenderer::render,
                ));

            let previous_anchor_count = heading_anchors.as_ref().map(|a| a.len()).unwrap_or(0);
            if let Some(anchors) = heading_anchors.as_mut() {
                viewer = viewer.heading_anchors(anchors);
            }

            let mut local_block_anchors = Vec::new();
            if block_anchors.is_some() {
                viewer = viewer.block_anchors(&mut local_block_anchors);
            }

            let mut computed_active_range: Option<std::ops::Range<usize>> = None;
            if let Some(global_line) = active_editor_line
                && global_line >= global_line_offset
                && global_line < global_line_offset + source_lines
            {
                let local_line = global_line - global_line_offset;
                computed_active_range = SectionRenderUtilsOps::byte_range_for_line(md, local_line);
            }

            if let Some(ref range) = computed_active_range {
                viewer = viewer.active_char_range(range.clone());
            }

            let resolved_hover = theme_colors
                .hover_bg
                .unwrap_or(crate::theme_bridge::TRANSPARENT);
            let resolved_active = theme_colors
                .active_bg
                .unwrap_or(crate::theme_bridge::TRANSPARENT);

            let list_highlight_fn = crate::widgets::MarkdownHooksOps::katana_list_item_highlight(
                computed_active_range.clone(),
                resolved_active,
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

            if let Some(idx) = search_active_index {
                viewer = viewer.search_active_match_index(idx);
            }

            let search_scroll_pending = ui.ctx().data(|d| {
                d.get_temp(egui::Id::new("katana_preview_search_scroll_pending"))
                    .unwrap_or(false)
            });
            viewer = viewer.search_scroll_pending(search_scroll_pending);

            let mut global_search_counter = ui.ctx().data(|d| {
                d.get_temp::<usize>(egui::Id::new("katana_preview_search_counter"))
                    .unwrap_or(0)
            });
            viewer = viewer.search_match_offset(&mut global_search_counter);

            viewer = viewer.search_query(search_query.clone());

            let safe_width = ui.available_width();
            let (_, newly_captured) = ui
                .scope(|ui| {
                    /* WHY: SYSTEMIC ARCHITECTURAL CONSTRAINT (Sandboxing)
                    Prevent egui_commonmark (which internally uses greedy ScrollAreas) from causing the
                    Ratchet Bug, where the preview pane's min_rect expands permanently.
                    By enforcing `set_max_width` at the root call, the inner scroll area's auto_shrink
                    demands are strictly capped to the currently available parent layout width. */
                    ui.set_max_width(safe_width);

                    if let Some(color) = theme_colors.text {
                        ui.visuals_mut().override_text_color = Some(color);
                    }
                    if let Some(border) = theme_colors.border {
                        ui.visuals_mut().widgets.noninteractive.bg_stroke.color = border;
                    }
                    const TABLE_STRIPE_ALPHA: f32 = 0.1;
                    if let Some(sel) = theme_colors.selection {
                        ui.visuals_mut().selection.bg_fill = sel;
                        ui.visuals_mut().faint_bg_color = sel.gamma_multiply(TABLE_STRIPE_ALPHA);
                    }
                    viewer.show_with_events(ui, cache, md)
                })
                .inner;

            ui.ctx().data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("katana_preview_search_counter"),
                    global_search_counter,
                )
            });

            anchors::MarkdownAnchorOps::process_anchors(
                md,
                global_line_offset,
                heading_anchors,
                block_anchors,
                hovered_lines,
                previous_anchor_count,
                local_block_anchors,
                local_hovered_spans,
            );

            tasks::MarkdownTaskOps::process_task_list_actions(
                md,
                newly_captured,
                global_task_list_idx,
                &mut actions,
            );
        });
        actions
    }
}
