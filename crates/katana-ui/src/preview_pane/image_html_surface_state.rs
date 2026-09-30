use super::{BrowserSessionAdapter, HtmlBrowserSurface};
use eframe::egui;

const RGB_COMPONENT_COUNT: usize = 3;
const RGBA_COMPONENT_COUNT: usize = 4;
const ALPHA_COMPONENT_INDEX: usize = RGB_COMPONENT_COUNT;
const OPAQUE_ALPHA: u8 = u8::MAX;

impl HtmlBrowserSurface {
    pub(super) fn document_origin(&self) -> Option<&str> {
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

    pub(super) fn resource_counts(&self) -> (usize, usize) {
        (
            usize::from(self.frame.is_some()),
            usize::from(self.texture.is_some()),
        )
    }

    pub(super) fn is_idle(&self) -> bool {
        self.adapter
            .as_ref()
            .is_some_and(BrowserSessionAdapter::is_idle)
    }

    #[cfg(test)]
    pub(super) fn wait_for_frame_for_test(
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
