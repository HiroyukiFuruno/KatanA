use crate::preview_pane::types::{SectionImageOps, SectionLogicOps};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalImageLoadingContext {
    InlineMarkdown,
    StandaloneDocument,
}

impl LocalImageLoadingContext {
    pub(crate) fn for_document_path(path: &std::path::Path) -> Self {
        /* WHY: 画像を埋め込んだ Markdown と、画像ファイル自体のプレビューを区別する。 */
        if katana_core::workspace::TreeEntry::path_is_image(path) {
            Self::StandaloneDocument
        } else {
            Self::InlineMarkdown
        }
    }

    pub(super) fn show_indicator(
        self,
        ui: &mut egui::Ui,
        presentation: &crate::widgets::LoadingPresentation,
    ) {
        let clipped_bounds = ui.clip_rect().intersect(ui.max_rect());
        if self == Self::StandaloneDocument && clipped_bounds.is_positive() {
            crate::widgets::LoadingIndicator::show_in_rect(ui, presentation, clipped_bounds);
        } else {
            crate::widgets::LoadingIndicator::show(ui, presentation);
        }
    }
}

impl SectionImageOps {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn handle_local_image_section(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        i: usize,
        loader: &crate::preview_pane::local_image_loader::LocalImageLoader,
        loading_context: LocalImageLoadingContext,
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
        let allow_controls =
            crate::preview_pane::types::SectionImageOps::interaction_controls_enabled(
                ui,
                is_slideshow,
            );
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
        if let Some(rect) = crate::preview_pane::ImageLogicOps::show_local_image_with_context(
            ui,
            path,
            i,
            loader,
            super::images::LocalImageDisplayOptions {
                loading_context,
                viewer_state: state,
                fullscreen_request: if !allow_controls {
                    None
                } else {
                    Some(&mut inner_req)
                },
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
                    #[cfg(feature = "screenshot-test-hooks")]
                    crate::preview_pane::overlay_inspection::PreviewOverlayInspectionOps::increment(
                        ui.ctx(),
                        |inspection| {
                            inspection.local_image_hover_background_renders += 1;
                        },
                    );
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
