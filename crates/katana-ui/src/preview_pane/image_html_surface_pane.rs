use super::{BrowserSessionAdapter, HtmlBrowserSurface};
use crate::preview_pane::types::PreviewPane;
use eframe::egui;

const RGB_COMPONENT_COUNT: usize = 3;
const RGBA_COMPONENT_COUNT: usize = 4;
const ALPHA_COMPONENT_INDEX: usize = RGB_COMPONENT_COUNT;
const OPAQUE_ALPHA: u8 = u8::MAX;

impl HtmlBrowserSurface {
    fn document_origin(&self) -> Option<&str> {
        self.document_origin.as_deref()
    }

    pub(super) fn display_rect(&self) -> Option<egui::Rect> {
        self.last_display_rect
    }

    pub(super) fn frame_matching_rgb_pixels(
        &self,
        expected: [u8; RGB_COMPONENT_COUNT],
    ) -> Option<u64> {
        self.frame.as_ref().map(|frame| {
            frame
                .pixels
                .as_chunks::<RGBA_COMPONENT_COUNT>()
                .0
                .iter()
                .filter(|pixel| {
                    pixel[0] == expected[0]
                        && pixel[1] == expected[1]
                        && pixel[2] == expected[2]
                        && pixel[ALPHA_COMPONENT_INDEX] == OPAQUE_ALPHA
                })
                .count() as u64
        })
    }

    pub(super) fn frame_viewport(&self) -> Option<(f32, f32)> {
        self.frame.as_ref().map(|frame| {
            (
                frame.viewport.logical_width(),
                frame.viewport.logical_height(),
            )
        })
    }

    pub(super) fn frame_generation(&self) -> Option<u64> {
        self.frame.as_ref().map(|frame| frame.generation)
    }

    pub(super) fn is_idle(&self) -> bool {
        self.adapter
            .as_ref()
            .is_some_and(BrowserSessionAdapter::is_idle)
    }

    #[cfg(test)]
    fn wait_for_frame_for_test(
        &mut self,
        ctx: &egui::Context,
        timeout: std::time::Duration,
    ) -> Result<(), String> {
        let deadline = std::time::Instant::now() + timeout;
        while self.frame.is_none() {
            if let Some(error) = &self.error {
                return Err(error.clone());
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let update = self
                .adapter
                .as_ref()
                .ok_or_else(|| "HTML browser adapter is not running".to_string())?
                .wait_for_update(remaining)
                .ok_or_else(|| "timed out waiting for the HTML browser frame".to_string())?;
            self.apply_update(ctx, update);
        }
        Ok(())
    }

    pub(super) fn frame_scroll_metrics(&self) -> Option<(f32, f32)> {
        self.frame
            .as_ref()
            .map(|frame| (frame.scroll_y, frame.content_height))
    }
}

impl PreviewPane {
    pub(crate) fn has_html_browser(&self) -> bool {
        self.html_browser.is_some()
    }

    pub(crate) fn html_browser_is_interacting(&self) -> bool {
        self.html_browser
            .as_ref()
            .is_some_and(HtmlBrowserSurface::is_interacting)
    }

    pub(crate) fn html_browser_origin(&self) -> Option<String> {
        self.html_browser
            .as_ref()
            .and_then(HtmlBrowserSurface::document_origin)
            .map(ToOwned::to_owned)
    }

    pub(crate) fn html_browser_navigation_history(&self) -> Option<Vec<String>> {
        self.html_browser
            .as_ref()
            .map(HtmlBrowserSurface::navigation_history)
    }

    pub(crate) fn html_browser_frame_matching_rgb_pixels(
        &self,
        expected: [u8; RGB_COMPONENT_COUNT],
    ) -> Option<u64> {
        self.html_browser
            .as_ref()
            .and_then(|browser| browser.frame_matching_rgb_pixels(expected))
    }

    pub(crate) fn html_browser_frame_viewport(&self) -> Option<(f32, f32)> {
        self.html_browser
            .as_ref()
            .and_then(HtmlBrowserSurface::frame_viewport)
    }

    pub(crate) fn html_browser_frame_generation(&self) -> Option<u64> {
        self.html_browser
            .as_ref()
            .and_then(HtmlBrowserSurface::frame_generation)
    }

    pub(crate) fn html_browser_is_idle(&self) -> Option<bool> {
        self.html_browser.as_ref().map(HtmlBrowserSurface::is_idle)
    }

    #[cfg(test)]
    pub(crate) fn wait_for_html_browser_frame_for_test(
        &mut self,
        ctx: &egui::Context,
        timeout: std::time::Duration,
    ) -> Result<(), String> {
        self.html_browser
            .as_mut()
            .ok_or_else(|| "active preview is not an HTML browser".to_string())?
            .wait_for_frame_for_test(ctx, timeout)
    }

    pub(crate) fn html_browser_frame_scroll_metrics(&self) -> Option<(f32, f32)> {
        self.html_browser
            .as_ref()
            .and_then(HtmlBrowserSurface::frame_scroll_metrics)
    }

    pub(crate) fn html_browser_display_rect(&self) -> Option<egui::Rect> {
        self.html_browser
            .as_ref()
            .and_then(HtmlBrowserSurface::display_rect)
    }

    pub(crate) fn dispatch_html_browser_input_burst_for_test(
        &mut self,
        count: u32,
    ) -> Result<(), String> {
        self.html_browser
            .as_mut()
            .ok_or_else(|| "active preview is not an HTML browser".to_string())?
            .dispatch_input_burst_for_test(count)
    }

    #[cfg(test)]
    pub(crate) fn start_html_browser_for_test(
        &mut self,
        viewport: katana_document_viewer::browser_session::HtmlBrowserViewport,
    ) -> bool {
        self.html_browser
            .as_mut()
            .is_some_and(|browser| browser.start_pending_session(viewport))
    }

    pub(crate) fn poll_html_browser(&mut self, ctx: &egui::Context) {
        if let Some(browser) = &mut self.html_browser {
            browser.poll(ctx);
        }
    }

    pub(crate) fn take_html_browser_navigation(&mut self) -> Option<String> {
        self.html_browser
            .as_mut()
            .and_then(|browser| browser.pending_navigation_urls.pop_front())
    }

    pub(crate) fn show_html_browser(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        self.html_browser
            .as_mut()
            .expect("HTML browser is present when its surface is shown")
            .show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::PreviewPane;
    use crate::preview_pane::image_html_surface::{BrowserFrame, HtmlBrowserSurface};

    #[test]
    fn queued_html_navigation_urls_are_taken_in_request_order() {
        let mut surface = HtmlBrowserSurface::start(
            katana_document_viewer::browser_session::HtmlBrowserSource::new(
                "<p>html</p>",
                "https://example.com/index.html",
            )
            .expect("browser source"),
        );
        surface
            .pending_navigation_urls
            .push_back("https://example.com/first".to_string());
        surface
            .pending_navigation_urls
            .push_back("https://example.com/second".to_string());

        let mut pane = PreviewPane::default();
        pane.html_browser = Some(surface);
        let first = pane.take_html_browser_navigation();
        let second = pane.take_html_browser_navigation();

        assert_eq!(first.as_deref(), Some("https://example.com/first"));
        assert_eq!(second.as_deref(), Some("https://example.com/second"));
        assert_eq!(pane.take_html_browser_navigation(), None);
    }

    #[test]
    fn browser_frame_generation_is_exposed_for_state_aware_harness_waits() {
        let mut pane = PreviewPane::default();
        assert_eq!(pane.html_browser_frame_generation(), None);

        let mut surface = HtmlBrowserSurface::failed("pending".to_string());
        let viewport = katana_document_viewer::browser_session::HtmlBrowserViewport::new(2, 1, 1.0)
            .expect("viewport");
        surface.frame = Some(BrowserFrame::new(7, viewport, 0.0, 1.0, vec![0; 8]));
        pane.html_browser = Some(surface);

        assert_eq!(pane.html_browser_frame_generation(), Some(7));
    }

    #[test]
    fn browser_idle_state_requires_an_active_adapter() {
        let mut pane = PreviewPane::default();
        assert_eq!(pane.html_browser_is_idle(), None);

        pane.html_browser = Some(HtmlBrowserSurface::failed("pending".to_string()));

        assert_eq!(pane.html_browser_is_idle(), Some(false));
    }
}
