use eframe::egui::{self, Pos2};
use katana_document_viewer::browser_session::{
    BrowserSessionAdapter, BrowserSessionRequest, BrowserSessionUpdate, HtmlBrowserSource,
    HtmlBrowserViewport,
};
use std::collections::VecDeque;

const FRAME_UPDATE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);
const FRAME_UPDATE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const HTML_BROWSER_ERROR_TEXT_PADDING: f32 = 12.0;

#[path = "image_html_surface_failure.rs"]
mod failure;
#[path = "image_html_surface_frame.rs"]
mod frame;
#[path = "image_html_surface_geometry.rs"]
mod geometry;
#[path = "image_html_surface_input.rs"]
mod input;
#[path = "image_html_surface_input_test_hooks.rs"]
mod input_test_hooks;
#[path = "image_html_surface_keyboard.rs"]
mod keyboard;
#[path = "image_html_surface_navigation.rs"]
mod navigation;
#[path = "image_html_surface_pane.rs"]
mod pane;
#[path = "image_html_surface_view.rs"]
mod view;

use frame::BrowserFrame;
use geometry::{frame_display_size, frame_position, frame_scroll_delta};

pub(crate) struct HtmlBrowserSurface {
    adapter: Option<BrowserSessionAdapter>,
    pending_source: Option<HtmlBrowserSource>,
    frame: Option<BrowserFrame>,
    document_origin: Option<String>,
    texture: Option<egui::TextureHandle>,
    viewport: Option<HtmlBrowserViewport>,
    last_pointer_position: Option<Pos2>,
    primary_pointer_pressed: bool,
    focused: bool,
    pointer_over: bool,
    last_display_rect: Option<egui::Rect>,
    error: Option<String>,
    frame_update_deadline: Option<std::time::Instant>,
    pending_navigation_urls: VecDeque<String>,
    navigation_history: VecDeque<String>,
}

impl HtmlBrowserSurface {
    pub(crate) fn start(source: HtmlBrowserSource) -> Self {
        let initial_origin = source.origin.as_str().to_owned();
        let document_origin = Some(initial_origin.clone());
        Self {
            adapter: None,
            pending_source: Some(source),
            frame: None,
            document_origin,
            texture: None,
            viewport: None,
            last_pointer_position: None,
            primary_pointer_pressed: false,
            focused: false,
            pointer_over: false,
            last_display_rect: None,
            error: None,
            frame_update_deadline: None,
            pending_navigation_urls: VecDeque::new(),
            navigation_history: VecDeque::from([initial_origin]),
        }
    }

    pub(crate) fn failed(error: String) -> Self {
        Self {
            adapter: None,
            pending_source: None,
            frame: None,
            document_origin: None,
            texture: None,
            viewport: None,
            last_pointer_position: None,
            primary_pointer_pressed: false,
            focused: false,
            pointer_over: false,
            last_display_rect: None,
            error: Some(error),
            frame_update_deadline: None,
            pending_navigation_urls: VecDeque::new(),
            navigation_history: VecDeque::new(),
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        loop {
            let update = self
                .adapter
                .as_ref()
                .and_then(BrowserSessionAdapter::take_update);
            let Some(update) = update else {
                break;
            };
            self.apply_update(ctx, update);
        }

        if self
            .frame_update_deadline
            .is_some_and(|deadline| std::time::Instant::now() < deadline)
        {
            ctx.request_repaint_after(FRAME_UPDATE_POLL_INTERVAL);
        } else {
            self.frame_update_deadline = None;
        }
    }

    fn apply_update(&mut self, ctx: &egui::Context, update: BrowserSessionUpdate) {
        match update {
            BrowserSessionUpdate::Frame(frame) => {
                if !self.accepts_frame_viewport(frame.viewport) {
                    return;
                }
                let origin = frame.origin.as_str().to_owned();
                self.record_navigation(origin.clone());
                self.document_origin = Some(origin);
                self.frame = Some(BrowserFrame::new(
                    frame.generation,
                    frame.viewport,
                    frame.scroll_y,
                    frame.content_height,
                    frame.pixels,
                ));
                self.update_texture(ctx);
                self.error = None;
                self.frame_update_deadline = None;
            }
            BrowserSessionUpdate::Navigation(navigation) => {
                self.pending_navigation_urls
                    .push_back(navigation.url.as_str().to_string());
            }
            BrowserSessionUpdate::Error(error) => {
                self.record_adapter_error("receive worker update", None, error);
            }
        }
    }

    fn await_frame(&mut self) {
        self.frame_update_deadline = Some(std::time::Instant::now() + FRAME_UPDATE_TIMEOUT);
    }

    fn record_error(&mut self, error: String) {
        self.frame = None;
        self.texture = None;
        self.last_pointer_position = None;
        self.primary_pointer_pressed = false;
        self.focused = false;
        self.pointer_over = false;
        self.error = Some(error);
        self.frame_update_deadline = None;
    }

    fn is_interacting(&self) -> bool {
        self.pointer_over || self.focused
    }

    fn accepts_frame_viewport(&self, frame_viewport: HtmlBrowserViewport) -> bool {
        self.viewport
            .is_none_or(|requested_viewport| requested_viewport == frame_viewport)
    }

    fn take_start_request(
        &mut self,
        viewport: HtmlBrowserViewport,
    ) -> Option<BrowserSessionRequest> {
        let source = self.pending_source.take()?;
        self.viewport = Some(viewport);
        Some(BrowserSessionRequest::new(source, viewport))
    }

    fn start_pending_session(&mut self, viewport: HtmlBrowserViewport) -> bool {
        let Some(request) = self.take_start_request(viewport) else {
            return false;
        };
        self.adapter = Some(BrowserSessionAdapter::start(request));
        self.await_frame();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::geometry::browser_viewport_for_ui;
    use super::*;
    use eframe::egui::Vec2;
    use katana_document_viewer::browser_session::BrowserSessionAdapterError;

    const DEVICE_SCALE_FACTOR: f32 = 2.0;
    const UI_WIDTH: f32 = 100.0;
    const UI_HEIGHT: f32 = 50.0;
    const FRAME_WIDTH: u32 = 200;
    const FRAME_HEIGHT: u32 = 100;
    const MAX_TEXTURE_SIDE: usize = 2048;

    #[test]
    fn browser_viewport_uses_physical_pixels_for_high_density_ui() {
        let viewport = browser_viewport_for_ui(
            Vec2::new(UI_WIDTH, UI_HEIGHT),
            DEVICE_SCALE_FACTOR,
            MAX_TEXTURE_SIDE,
        );

        assert_eq!(
            viewport,
            HtmlBrowserViewport::new(FRAME_WIDTH, FRAME_HEIGHT, DEVICE_SCALE_FACTOR).ok()
        );
    }

    #[test]
    fn browser_viewport_preserves_logical_size_within_texture_limits() -> Result<(), String> {
        let logical = Vec2::new(1079.0, 642.5);
        let viewport = browser_viewport_for_ui(logical, DEVICE_SCALE_FACTOR, MAX_TEXTURE_SIDE)
            .ok_or_else(|| "limited browser viewport was not created".to_string())?;

        assert!(viewport.width <= MAX_TEXTURE_SIDE as u32);
        assert!(viewport.height <= MAX_TEXTURE_SIDE as u32);
        assert!((viewport.logical_width() - logical.x).abs() <= 0.5);
        assert!((viewport.logical_height() - logical.y).abs() <= 0.5);
        Ok(())
    }

    #[test]
    fn frame_display_size_returns_logical_ui_size() -> Result<(), String> {
        let viewport = HtmlBrowserViewport::new(FRAME_WIDTH, FRAME_HEIGHT, DEVICE_SCALE_FACTOR)
            .map_err(|error| error.to_string())?;

        assert_eq!(frame_display_size(viewport), Vec2::new(UI_WIDTH, UI_HEIGHT));
        Ok(())
    }

    #[test]
    fn frame_coordinates_follow_the_displayed_frame_scale() -> Result<(), String> {
        let viewport = HtmlBrowserViewport::new(FRAME_WIDTH, FRAME_HEIGHT, DEVICE_SCALE_FACTOR)
            .map_err(|error| error.to_string())?;
        let rect =
            egui::Rect::from_min_size(egui::pos2(10.0, 20.0), Vec2::new(UI_WIDTH, UI_HEIGHT));

        assert_eq!(
            frame_position(rect, egui::pos2(60.0, 45.0), viewport),
            egui::pos2(100.0, 50.0)
        );
        assert_eq!(
            frame_scroll_delta(rect, Vec2::new(5.0, 2.5), viewport),
            Vec2::new(10.0, 5.0)
        );
        Ok(())
    }

    #[test]
    fn failed_surface_preserves_typed_error_without_starting_a_fallback_session() {
        let mut surface = HtmlBrowserSurface::failed("invalid document origin".to_string());

        surface.poll(&egui::Context::default());

        assert!(surface.adapter.is_none());
        assert_eq!(surface.error.as_deref(), Some("invalid document origin"));
        assert!(surface.frame.is_none());
        assert!(surface.frame_update_deadline.is_none());
    }

    #[test]
    fn runtime_error_discards_the_stale_browser_frame() {
        let viewport = HtmlBrowserViewport::new(1, 1, 1.0).unwrap();
        let mut surface = HtmlBrowserSurface::failed("initial".to_string());
        surface.frame = Some(BrowserFrame::new(1, viewport, 0.0, 1.0, vec![0, 0, 0, 255]));
        surface.last_pointer_position = Some(egui::pos2(1.0, 1.0));
        surface.primary_pointer_pressed = true;
        surface.focused = true;
        surface.pointer_over = true;

        surface.record_error("runtime failed".to_string());

        assert!(surface.frame.is_none());
        assert!(surface.texture.is_none());
        assert!(surface.last_pointer_position.is_none());
        assert!(!surface.primary_pointer_pressed);
        assert!(!surface.focused);
        assert!(!surface.pointer_over);
        assert_eq!(surface.error.as_deref(), Some("runtime failed"));
    }

    #[test]
    fn worker_stop_does_not_overwrite_the_primary_browser_error() {
        let mut surface = HtmlBrowserSurface::failed("primary KRR failure".to_string());

        surface.record_adapter_error(
            "resize",
            Some("file:///workspace/index.html".to_string()),
            BrowserSessionAdapterError::WorkerStopped,
        );

        assert_eq!(surface.error.as_deref(), Some("primary KRR failure"));
    }

    #[test]
    fn adapter_errors_include_layer_operation_document_and_cause() {
        let mut surface = HtmlBrowserSurface::failed("initial".to_string());
        surface.error = None;

        surface.record_adapter_error(
            "resize",
            Some("file:///workspace/index.html".to_string()),
            BrowserSessionAdapterError::CommandQueueFull,
        );

        let error = surface.error.as_deref().unwrap_or_default();
        assert!(error.contains("Layer: KDV worker"));
        assert!(error.contains("Operation: resize"));
        assert!(error.contains("Document: file:///workspace/index.html"));
        assert!(error.contains("Cause: browser command queue is full"));
    }

    #[test]
    fn lifecycle_script_error_does_not_replace_the_rendered_surface() {
        let _runtime_guard = crate::preview_pane::html_browser_runtime_test_guard();
        let source = HtmlBrowserSource::new(
            "<script>document.addEventListener('DOMContentLoaded', () => { throw new Error('lifecycle failed'); });</script>",
            "https://example.test/index.html",
        )
        .expect("valid browser source");
        let mut surface = HtmlBrowserSurface::start(source);
        assert!(
            surface
                .start_pending_session(HtmlBrowserViewport::new(320, 240, 1.0).expect("viewport"))
        );
        let context = egui::Context::default();
        let update = surface
            .adapter
            .as_ref()
            .expect("browser adapter")
            .wait_for_update(std::time::Duration::from_secs(10))
            .expect("worker update");
        surface.apply_update(&context, update);

        assert!(surface.error.is_none());
        let frame = surface.frame.as_ref().expect("rendered browser frame");
        assert_eq!(frame.viewport.width, 320);
        assert_eq!(frame.viewport.height, 240);
        assert!(!frame.pixels.is_empty());
        assert!(surface.texture.is_some());
    }

    #[test]
    fn browser_session_start_waits_for_the_real_ui_viewport() {
        let source =
            HtmlBrowserSource::new("<p>Initial</p>", "https://example.com/initial").unwrap();
        let requested = HtmlBrowserViewport::new(320, 240, 1.0).unwrap();
        let mut surface = HtmlBrowserSurface::start(source);

        assert!(surface.adapter.is_none());
        assert!(surface.viewport.is_none());
        assert!(surface.frame_update_deadline.is_none());
        let request = surface
            .take_start_request(requested)
            .expect("pending source must create one request");

        assert_eq!(request.viewport, requested);
        assert_eq!(
            request.source.origin.as_str(),
            "https://example.com/initial"
        );
        assert!(surface.take_start_request(requested).is_none());
        assert!(surface.accepts_frame_viewport(requested));
    }

    #[test]
    fn raw_browser_frame_rgb_count_is_available_without_texture_sampling() {
        let viewport = HtmlBrowserViewport::new(2, 1, 1.0).unwrap();
        let mut surface = HtmlBrowserSurface::failed("test".to_string());
        surface.frame = Some(BrowserFrame::new(
            1,
            viewport,
            0.0,
            1.0,
            vec![232, 199, 255, 255, 0, 0, 0, 255],
        ));

        assert_eq!(surface.frame_matching_rgb_pixels([232, 199, 255]), Some(1));
        assert_eq!(surface.frame_matching_rgb_pixels([1, 2, 3]), Some(0));
        surface.frame = Some(BrowserFrame::new(
            2,
            viewport,
            0.0,
            1.0,
            vec![232, 199, 255, 255, 7, 8],
        ));
        assert_eq!(surface.frame_matching_rgb_pixels([232, 199, 255]), Some(1));
        surface.frame = None;
        assert_eq!(surface.frame_matching_rgb_pixels([232, 199, 255]), None);
    }

    #[test]
    fn displayed_surface_rect_is_available_for_headless_targeting() {
        let mut surface = HtmlBrowserSurface::failed("test".to_string());
        let rect = egui::Rect::from_min_size(egui::pos2(12.0, 24.0), egui::vec2(320.0, 180.0));

        assert_eq!(surface.display_rect(), None);
        surface.last_display_rect = Some(rect);
        assert_eq!(surface.display_rect(), Some(rect));
    }

    #[test]
    fn navigation_before_first_layout_replaces_the_pending_source_and_records_history() {
        let initial = HtmlBrowserSource::new("<p>Initial</p>", "https://example.com/initial")
            .expect("initial source");
        let next =
            HtmlBrowserSource::new("<p>Next</p>", "https://example.com/next").expect("next source");
        let mut surface = HtmlBrowserSurface::start(initial);

        surface.navigate(next);

        assert!(surface.adapter.is_none());
        assert_eq!(
            surface
                .pending_source
                .as_ref()
                .map(|source| source.origin.as_str()),
            Some("https://example.com/next")
        );
        assert_eq!(
            surface.navigation_history(),
            [
                "https://example.com/initial".to_owned(),
                "https://example.com/next".to_owned()
            ]
        );
    }
}
