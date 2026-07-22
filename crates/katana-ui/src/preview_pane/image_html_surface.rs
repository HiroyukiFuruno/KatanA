use eframe::egui::{self, Pos2, Vec2};
use katana_document_viewer::browser_session::{
    BrowserSessionAdapter, BrowserSessionRequest, BrowserSessionUpdate, HtmlBrowserSource,
    HtmlBrowserViewport,
};
use std::collections::VecDeque;

const INITIAL_VIEWPORT_DIMENSION: u32 = 1;
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
#[path = "image_html_surface_keyboard.rs"]
mod keyboard;
#[path = "image_html_surface_navigation.rs"]
mod navigation;
#[path = "image_html_surface_pane.rs"]
mod pane;
#[path = "image_html_surface_view.rs"]
mod view;

use frame::BrowserFrame;
use geometry::{browser_viewport_for_ui, frame_display_size, frame_position, frame_scroll_delta};

pub(crate) struct HtmlBrowserSurface {
    adapter: Option<BrowserSessionAdapter>,
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
    pending_navigation_url: Option<String>,
    navigation_history: VecDeque<String>,
}

impl HtmlBrowserSurface {
    pub(crate) fn start(source: HtmlBrowserSource) -> Self {
        let initial_origin = source.origin.as_str().to_owned();
        let document_origin = Some(initial_origin.clone());
        let viewport =
            HtmlBrowserViewport::new(INITIAL_VIEWPORT_DIMENSION, INITIAL_VIEWPORT_DIMENSION, 1.0)
                .expect("constant initial browser viewport is valid");
        Self {
            adapter: Some(BrowserSessionAdapter::start(BrowserSessionRequest::new(
                source, viewport,
            ))),
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
            frame_update_deadline: Some(std::time::Instant::now() + FRAME_UPDATE_TIMEOUT),
            pending_navigation_url: None,
            navigation_history: VecDeque::from([initial_origin]),
        }
    }

    pub(crate) fn failed(error: String) -> Self {
        Self {
            adapter: None,
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
            pending_navigation_url: None,
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
            match update {
                BrowserSessionUpdate::Frame(frame) => {
                    if !self.accepts_frame_viewport(frame.viewport) {
                        continue;
                    }
                    let origin = frame.origin.as_str().to_owned();
                    self.record_navigation(origin.clone());
                    self.document_origin = Some(origin);
                    self.frame = Some(BrowserFrame::new(
                        frame.generation,
                        frame.viewport,
                        frame.pixels,
                    ));
                    self.update_texture(ctx);
                    self.error = None;
                    self.frame_update_deadline = None;
                }
                BrowserSessionUpdate::Navigation(navigation) => {
                    self.pending_navigation_url = Some(navigation.url.as_str().to_string());
                }
                BrowserSessionUpdate::Error(error) => {
                    self.record_adapter_error("receive worker update", None, error);
                }
            }
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

    fn resize_to_ui(&mut self, ui: &egui::Ui) {
        let Some(adapter) = &self.adapter else {
            return;
        };
        let size = ui.available_size().max(Vec2::splat(1.0));
        let Some(viewport) = browser_viewport_for_ui(size, ui.ctx().pixels_per_point()) else {
            return;
        };
        if self.viewport == Some(viewport) {
            return;
        }
        if let Err(error) = adapter.resize(viewport) {
            self.record_adapter_error("resize", None, error);
            return;
        }
        self.discard_bootstrap_frame(viewport);
        self.viewport = Some(viewport);
        self.await_frame();
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

    fn discard_bootstrap_frame(&mut self, requested_viewport: HtmlBrowserViewport) {
        let is_bootstrap = self.frame.as_ref().is_some_and(|frame| {
            frame.viewport.width == INITIAL_VIEWPORT_DIMENSION
                && frame.viewport.height == INITIAL_VIEWPORT_DIMENSION
                && frame.viewport != requested_viewport
        });
        if is_bootstrap {
            self.frame = None;
            self.texture = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_document_viewer::browser_session::BrowserSessionAdapterError;

    const DEVICE_SCALE_FACTOR: f32 = 2.0;
    const UI_WIDTH: f32 = 100.0;
    const UI_HEIGHT: f32 = 50.0;
    const FRAME_WIDTH: u32 = 200;
    const FRAME_HEIGHT: u32 = 100;

    #[test]
    fn browser_viewport_uses_physical_pixels_for_high_density_ui() {
        let viewport = browser_viewport_for_ui(Vec2::new(UI_WIDTH, UI_HEIGHT), DEVICE_SCALE_FACTOR);

        assert_eq!(
            viewport,
            HtmlBrowserViewport::new(FRAME_WIDTH, FRAME_HEIGHT, DEVICE_SCALE_FACTOR).ok()
        );
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
        surface.frame = Some(BrowserFrame::new(1, viewport, vec![0, 0, 0, 255]));
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
    fn lifecycle_script_error_reaches_the_surface_with_runtime_stack() {
        let source = HtmlBrowserSource::new(
            "<script>document.addEventListener('DOMContentLoaded', () => { throw new Error('lifecycle failed'); });</script>",
            "https://example.test/index.html",
        )
        .expect("valid browser source");
        let mut surface = HtmlBrowserSurface::start(source);
        let context = egui::Context::default();
        let deadline = std::time::Instant::now() + FRAME_UPDATE_TIMEOUT;

        while surface.error.is_none() && std::time::Instant::now() < deadline {
            surface.poll(&context);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let error = surface.error.as_deref().unwrap_or_default();
        for expected in [
            "Layer: KDV worker",
            "Operation: receive worker update",
            "Layer: KRR runtime",
            "Operation: start",
            "Document: https://example.test/index.html",
            "Cause: in-process HTML runtime failed",
            "JavaScript exception: Error: lifecycle failed",
            "inline-script:1:",
            "krr-html-dom-bootstrap",
        ] {
            assert!(error.contains(expected), "missing {expected:?} in {error}");
        }
    }

    #[test]
    fn requested_viewport_rejects_stale_bootstrap_frames() {
        let bootstrap = HtmlBrowserViewport::new(1, 1, 1.0).unwrap();
        let requested = HtmlBrowserViewport::new(320, 240, 1.0).unwrap();
        let mut surface = HtmlBrowserSurface::failed("test".to_string());
        surface.frame = Some(BrowserFrame::new(1, bootstrap, vec![255, 255, 255, 255]));

        assert!(surface.accepts_frame_viewport(bootstrap));
        surface.discard_bootstrap_frame(requested);
        surface.viewport = Some(requested);

        assert!(surface.frame.is_none());
        assert!(!surface.accepts_frame_viewport(bootstrap));
        assert!(surface.accepts_frame_viewport(requested));
    }

    #[test]
    fn raw_browser_frame_rgb_count_is_available_without_texture_sampling() {
        let viewport = HtmlBrowserViewport::new(2, 1, 1.0).unwrap();
        let mut surface = HtmlBrowserSurface::failed("test".to_string());
        surface.frame = Some(BrowserFrame::new(
            1,
            viewport,
            vec![232, 199, 255, 255, 0, 0, 0, 255],
        ));

        assert_eq!(surface.frame_matching_rgb_pixels([232, 199, 255]), Some(1));
        assert_eq!(surface.frame_matching_rgb_pixels([1, 2, 3]), Some(0));
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
    fn navigation_reuses_the_adapter_and_records_tab_history() {
        let initial = HtmlBrowserSource::new("<p>Initial</p>", "https://example.com/initial")
            .expect("initial source");
        let next =
            HtmlBrowserSource::new("<p>Next</p>", "https://example.com/next").expect("next source");
        let mut surface = HtmlBrowserSurface::start(initial);
        let adapter_address = surface
            .adapter
            .as_ref()
            .map(|adapter| std::ptr::from_ref(adapter).addr());

        surface.navigate(next);

        assert_eq!(
            surface
                .adapter
                .as_ref()
                .map(|adapter| std::ptr::from_ref(adapter).addr()),
            adapter_address
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
