use crate::preview_pane::{RenderedSection, ViewerState};
use eframe::egui;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const FULLSCREEN_PADDING: f32 = 40.0;
pub(crate) const FULLSCREEN_CLOSE_SIZE: f32 = 32.0;
pub(crate) const FULLSCREEN_CLOSE_MARGIN: f32 = 20.0;
pub(crate) const MIN_ZOOM: f32 = 0.5;
pub(crate) const MAX_ZOOM: f32 = 5.0;

pub(super) const SLIDESHOW_CONTROL_HEIGHT: f32 = 36.0;
pub(super) const SLIDESHOW_CONTROL_WIDTH: f32 = 160.0;
pub(super) const SLIDESHOW_CONTROL_CORNER_RADIUS: f32 = 18.0;
pub(super) const SLIDESHOW_CONTROL_PADDING_X: i8 = 20;
pub(super) const SLIDESHOW_CONTROL_PADDING_Y: i8 = 4;
pub(super) const SLIDESHOW_CONTROL_SPACING: f32 = 8.0;
pub(super) const SLIDESHOW_BG_ALPHA_SCALE: f32 = 0.8;

pub(super) const SLIDESHOW_CONTROL_FADE_DELAY: f64 = 1.5;
pub(super) const SLIDESHOW_CONTROL_FADE_DURATION: f32 = 0.5;
pub(super) const SLIDESHOW_OPACITY_MAX: f32 = 1.0;
pub(super) const SLIDESHOW_OPACITY_MIN: f32 = 0.0;

pub use super::types::FullscreenLogicOps;

impl FullscreenLogicOps {
    pub(crate) fn render_fullscreen_if_active(
        ctx: &egui::Context,
        sections: &[RenderedSection],
        fullscreen_image: Option<usize>,
        fullscreen_state: &mut ViewerState,
    ) -> Option<usize> {
        let idx = fullscreen_image?;
        if let Some(RenderedSection::Image { svg_data, alt, .. }) = sections.get(idx) {
            if Self::show_fullscreen_modal(ctx, svg_data, alt, fullscreen_state, idx) {
                Some(idx) /* WHY: keep open */
            } else {
                None /* WHY: user closed */
            }
        } else if let Some(RenderedSection::LocalImage { path, alt, .. }) = sections.get(idx) {
            if Self::show_fullscreen_local_image(ctx, path, alt, fullscreen_state, idx) {
                Some(idx) /* WHY: keep open */
            } else {
                None /* WHY: user closed */
            }
        } else {
            None /* WHY: section gone */
        }
    }

    pub(crate) fn show_fullscreen_modal(
        ctx: &egui::Context,
        img: &RasterizedSvg,
        _alt: &str,
        viewer_state: &mut ViewerState,
        idx: usize,
    ) -> bool {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return false;
        }
        let dc_close = crate::i18n::I18nOps::get()
            .preview
            .diagram_controller
            .close
            .clone();
        super::fullscreen_svg::show_fullscreen_svg(ctx, img, &dc_close, viewer_state, idx)
    }

    pub(crate) fn show_fullscreen_local_image(
        ctx: &egui::Context,
        path: &std::path::Path,
        _alt: &str,
        viewer_state: &mut ViewerState,
        idx: usize,
    ) -> bool {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return false;
        }
        let dc_close = crate::i18n::I18nOps::get()
            .preview
            .diagram_controller
            .close
            .clone();
        super::fullscreen_local::show_fullscreen_local(ctx, path, &dc_close, viewer_state, idx)
    }

    pub(crate) fn render_slideshow_modal(
        ctx: &egui::Context,
        layout: &mut crate::state::layout::LayoutState,
        pane: &mut crate::preview_pane::PreviewPane,
    ) {
        super::slideshow::render_slideshow_modal(ctx, layout, pane);
    }
}
