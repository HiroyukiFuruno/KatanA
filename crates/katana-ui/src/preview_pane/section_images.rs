use crate::preview_pane::types::{SectionImageOps, SectionLogicOps};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

impl SectionImageOps {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn handle_image_section(
        ui: &mut egui::Ui,
        svg_data: &RasterizedSvg,
        alt: &str,
        i: usize,
        lines_in_section: usize,
        global_line_offset: usize,
        active_editor_line: Option<usize>,
        viewer_states: Option<&mut Vec<crate::preview_pane::ViewerState>>,
        fullscreen_request: Option<&mut Option<usize>>,
        section_lifecycle: &mut Option<&mut Vec<crate::preview_pane::SectionLifecycle>>,
        block_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
        is_slideshow: bool,
    ) {
        let allow_controls = !is_slideshow
            || ui.ctx().data(|d| {
                d.get_temp(egui::Id::new("katana_slideshow_diagram_controls"))
                    .unwrap_or(false)
            });
        let allow_hover = !is_slideshow
            || ui.ctx().data(|d| {
                d.get_temp(egui::Id::new("katana_slideshow_hover_highlight"))
                    .unwrap_or(false)
            });

        /* WHY: In slideshow mode diagrams are read-only; controls and hover highlight are hidden by default */
        let state = if !allow_controls {
            None
        } else {
            viewer_states.map(|vs| {
                if vs.len() <= i {
                    vs.resize_with(i + 1, crate::preview_pane::ViewerState::default);
                }
                &mut vs[i]
            })
        };

        let is_active = !is_slideshow
            && active_editor_line.is_some_and(|line| {
                line >= global_line_offset && line < global_line_offset + lines_in_section
            });

        let rect = crate::preview_pane::ImageLogicOps::show_rasterized(
            ui,
            svg_data,
            alt,
            i,
            state,
            if !allow_controls {
                None
            } else {
                fullscreen_request
            },
            |ui, rect, is_hovered| {
                if allow_hover && (is_hovered || is_active) {
                    let tc = ui.ctx().data(|d| {
                        d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                            "katana_theme_colors",
                        ))
                    });
                    if let Some(tc) = tc {
                        let color = crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                            tc.preview.hover_line_background,
                        );
                        let mut highlight_rect = rect;
                        highlight_rect.min.x = ui.max_rect().min.x;
                        highlight_rect.max.x = ui.max_rect().max.x;
                        ui.painter().rect_filled(highlight_rect, 0.0, color);
                    }
                }
            },
        );
        SectionLogicOps::mark_drawn_and_anchor(
            rect,
            i,
            lines_in_section,
            global_line_offset,
            section_lifecycle,
            block_anchors,
        );
        if ui.rect_contains_pointer(rect)
            && let Some(hovered) = hovered_lines
        {
            hovered.push(global_line_offset..global_line_offset + lines_in_section);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn handle_local_image_section(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        alt: &str,
        i: usize,
        lines_in_section: usize,
        global_line_offset: usize,
        active_editor_line: Option<usize>,
        viewer_states: Option<&mut Vec<crate::preview_pane::ViewerState>>,
        fullscreen_request: Option<&mut Option<usize>>,
        section_lifecycle: &mut Option<&mut Vec<crate::preview_pane::SectionLifecycle>>,
        block_anchors: &mut Option<&mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
        hovered_lines: Option<&mut Vec<std::ops::Range<usize>>>,
        is_slideshow: bool,
    ) {
        let allow_controls = !is_slideshow
            || ui.ctx().data(|d| {
                d.get_temp(egui::Id::new("katana_slideshow_diagram_controls"))
                    .unwrap_or(false)
            });
        let allow_hover = !is_slideshow
            || ui.ctx().data(|d| {
                d.get_temp(egui::Id::new("katana_slideshow_hover_highlight"))
                    .unwrap_or(false)
            });

        /* WHY: Controls and hover hidden in slideshow by default */
        let state = if !allow_controls {
            None
        } else {
            viewer_states.map(|vs| {
                if vs.len() <= i {
                    vs.resize_with(i + 1, crate::preview_pane::ViewerState::default);
                }
                &mut vs[i]
            })
        };

        let is_active = !is_slideshow
            && active_editor_line.is_some_and(|line| {
                line >= global_line_offset && line < global_line_offset + lines_in_section
            });

        let mut inner_req = None;
        if let Some(rect) = crate::preview_pane::ImageLogicOps::show_local_image(
            ui,
            path,
            alt,
            i,
            state,
            if !allow_controls {
                None
            } else {
                Some(&mut inner_req)
            },
            |ui, rect, is_hovered| {
                if allow_hover
                    && (is_hovered || is_active)
                    && let Some(tc) = ui.ctx().data(|d| {
                        d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                            "katana_theme_colors",
                        ))
                    })
                {
                    let color = crate::theme_bridge::ThemeBridgeOps::rgba_to_color32(
                        tc.preview.hover_line_background,
                    );
                    let mut highlight_rect = rect;
                    highlight_rect.min.x = ui.max_rect().min.x;
                    highlight_rect.max.x = ui.max_rect().max.x;
                    ui.painter().rect_filled(highlight_rect, 0.0, color);
                }
            },
        ) {
            if let Some(req) = inner_req
                && let Some(outer) = fullscreen_request
            {
                *outer = Some(req);
            }
            SectionLogicOps::mark_drawn_and_anchor(
                rect,
                i,
                lines_in_section,
                global_line_offset,
                section_lifecycle,
                block_anchors,
            );
            if ui.rect_contains_pointer(rect)
                && let Some(hovered) = hovered_lines
            {
                hovered.push(global_line_offset..global_line_offset + lines_in_section);
            }
        }
    }
}
