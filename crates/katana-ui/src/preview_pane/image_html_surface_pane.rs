use super::HtmlBrowserSurface;
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
                .chunks_exact(RGBA_COMPONENT_COUNT)
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
    use crate::preview_pane::image_html_surface::HtmlBrowserSurface;

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
}
