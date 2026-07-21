use crate::preview_pane::{RenderedSection, ViewerState};
use eframe::egui;
use katana_core::markdown::svg_rasterize::RasterizedSvg;

pub(crate) const FULLSCREEN_PADDING: f32 = 40.0;
pub(crate) const FULLSCREEN_CLOSE_SIZE: f32 = 32.0;
pub(crate) const FULLSCREEN_CLOSE_MARGIN: f32 = 20.0;
pub(crate) const MIN_ZOOM: f32 = 0.5;
pub(crate) const MAX_ZOOM: f32 = 5.0;

const CLOSE_CONTROL_BG_ALPHA: u8 = 0xb8;
const CLOSE_CONTROL_BORDER_ALPHA: u8 = 0xc8;
const CLOSE_CONTROL_ICON_ALPHA: u8 = 0xf0;

pub(super) const SLIDESHOW_CONTROL_HEIGHT: f32 = 36.0;
pub(super) const SLIDESHOW_CONTROL_WIDTH: f32 = 160.0;
pub(super) const SLIDESHOW_CONTROL_CORNER_RADIUS: f32 = 18.0;
pub(super) const SLIDESHOW_CONTROL_PADDING_X: i8 = 20;
pub(super) const SLIDESHOW_CONTROL_PADDING_Y: i8 = 4;
pub(super) const SLIDESHOW_CONTROL_SPACING: f32 = 8.0;
pub(super) const SLIDESHOW_BG_ALPHA_SCALE: f32 = 1.0;

pub(super) const SLIDESHOW_CONTROL_FADE_DELAY: f64 = 1.5;
pub(super) const SLIDESHOW_CONTROL_FADE_DURATION: f32 = 0.5;
pub(super) const SLIDESHOW_OPACITY_MAX: f32 = 1.0;
pub(super) const SLIDESHOW_OPACITY_MIN: f32 = 0.0;

pub(crate) const FULLSCREEN_FADE_OUT_DURATION: f32 = 0.2;

pub use super::types::FullscreenLogicOps;

pub(super) fn render_fullscreen_close_button(
    ui: &mut egui::Ui,
    blocker_rect: egui::Rect,
    hover_text: &str,
    opacity: f32,
) -> bool {
    let close_button_size = egui::Vec2::splat(FULLSCREEN_CLOSE_SIZE);
    let close_button_rect = egui::Rect::from_min_size(
        egui::pos2(
            blocker_rect.right() - close_button_size.x - FULLSCREEN_CLOSE_MARGIN,
            blocker_rect.top() + FULLSCREEN_CLOSE_MARGIN,
        ),
        close_button_size,
    );
    let (background, border, icon_color) = fullscreen_close_control_style(opacity);

    ui.put(
        close_button_rect,
        egui::Button::image(
            crate::icon::Icon::CloseModal
                .image(crate::icon::IconSize::Large)
                .tint(icon_color),
        )
        .fill(background)
        .stroke(border),
    )
    .on_hover_text(hover_text)
    .on_hover_cursor(egui::CursorIcon::PointingHand)
    .clicked()
}

fn fullscreen_close_control_style(opacity: f32) -> (egui::Color32, egui::Stroke, egui::Color32) {
    let opacity = opacity.clamp(0.0, 1.0);
    (
        crate::theme_bridge::ThemeBridgeOps::from_black_alpha(CLOSE_CONTROL_BG_ALPHA)
            .gamma_multiply(opacity),
        egui::Stroke::new(
            1.0,
            crate::theme_bridge::ThemeBridgeOps::from_white_alpha(CLOSE_CONTROL_BORDER_ALPHA)
                .gamma_multiply(opacity),
        ),
        crate::theme_bridge::ThemeBridgeOps::from_white_alpha(CLOSE_CONTROL_ICON_ALPHA)
            .gamma_multiply(opacity),
    )
}

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
                /* WHY: keep open */
                Some(idx)
            } else {
                /* WHY: user closed */
                None
            }
        } else if let Some(RenderedSection::LocalImage { path, alt, .. }) = sections.get(idx) {
            if Self::show_fullscreen_local_image(ctx, path, alt, fullscreen_state, idx) {
                /* WHY: keep open */
                Some(idx)
            } else {
                /* WHY: user closed */
                None
            }
        } else {
            /* WHY: section gone */
            None
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
        super::slideshow::SlideshowModalOps::render_slideshow_modal(ctx, layout, pane);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FullscreenInteraction {
    hovered: bool,
    dragged: bool,
    zoom_delta: f32,
    drag_delta: egui::Vec2,
    smooth_scroll_delta: egui::Vec2,
}

impl FullscreenInteraction {
    pub(crate) fn from_input(ui: &egui::Ui, response: &egui::Response) -> Self {
        let (zoom_delta, smooth_scroll_delta) =
            ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta));

        Self {
            hovered: response.hovered(),
            dragged: response.dragged(),
            zoom_delta,
            drag_delta: response.drag_delta(),
            smooth_scroll_delta,
        }
    }
}

impl FullscreenInteraction {
    pub(crate) fn apply(self, viewer_state: &mut ViewerState) {
        if !self.hovered || viewer_state.closing_since.is_some() {
            return;
        }

        if self.smooth_scroll_delta == egui::Vec2::ZERO && self.zoom_delta != 1.0 {
            viewer_state.zoom = (viewer_state.zoom * self.zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
        }

        viewer_state.pan += if self.dragged {
            self.drag_delta
        } else {
            self.smooth_scroll_delta
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullscreen_close_control_has_fixed_background_and_border() {
        let (background, border, icon) = fullscreen_close_control_style(1.0);
        let [background_r, background_g, background_b, background_a] =
            background.to_srgba_unmultiplied();
        let [border_r, border_g, border_b, border_a] = border.color.to_srgba_unmultiplied();
        let [icon_r, icon_g, icon_b, icon_a] = icon.to_srgba_unmultiplied();

        assert_eq!((background_r, background_g, background_b), (0, 0, 0));
        assert!(background_a >= 0xb0);
        assert_eq!(border.width, 1.0);
        assert_eq!((border_r, border_g, border_b), (255, 255, 255));
        assert!(border_a >= 0xc0);
        assert_eq!((icon_r, icon_g, icon_b), (255, 255, 255));
        assert!(icon_a >= 0xf0);
    }

    #[test]
    fn fullscreen_interaction_preserves_zoom_on_scroll() {
        let mut viewer_state = ViewerState::default();
        viewer_state.zoom_in();
        viewer_state.zoom_in();
        let zoom_before = viewer_state.zoom;

        FullscreenInteraction {
            hovered: true,
            dragged: false,
            zoom_delta: 0.6,
            drag_delta: egui::Vec2::ZERO,
            smooth_scroll_delta: egui::vec2(0.0, 8.0),
        }
        .apply(&mut viewer_state);

        assert_eq!(viewer_state.zoom, zoom_before);
        assert_eq!(viewer_state.pan, egui::vec2(0.0, 8.0));
    }

    #[test]
    fn fullscreen_interaction_zooms_only_when_not_scrolling() {
        let mut viewer_state = ViewerState::default();
        let expected_zoom = (viewer_state.zoom * 1.2).clamp(MIN_ZOOM, MAX_ZOOM);

        FullscreenInteraction {
            hovered: true,
            dragged: false,
            zoom_delta: 1.2,
            drag_delta: egui::Vec2::ZERO,
            smooth_scroll_delta: egui::Vec2::ZERO,
        }
        .apply(&mut viewer_state);

        assert_eq!(viewer_state.zoom, expected_zoom);
        assert_eq!(viewer_state.pan, egui::Vec2::ZERO);
    }
}
