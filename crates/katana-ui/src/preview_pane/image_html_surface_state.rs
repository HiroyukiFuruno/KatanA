use super::{BrowserSessionAdapter, HtmlBrowserSurface};
use eframe::egui;

const RGB_COMPONENT_COUNT: usize = 3;
const RGBA_COMPONENT_COUNT: usize = 4;
const ALPHA_COMPONENT_INDEX: usize = RGB_COMPONENT_COUNT;
const OPAQUE_ALPHA: u8 = u8::MAX;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HtmlReceiptObservation {
    pub(crate) source_uri: String,
    pub(crate) katana_adapter_instance_id: u64,
    pub(crate) katana_renderer_session_epoch: u64,
    pub(crate) katana_frame_renderer_session_epoch: u64,
    pub(crate) frame_generation: u64,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) scroll_y: f32,
    pub(crate) content_height: f32,
    pub(crate) idle: bool,
}

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

    pub(super) fn receipt_observation(&self) -> Result<Option<HtmlReceiptObservation>, String> {
        let Some((katana_adapter_instance_id, katana_renderer_session_epoch)) =
            self.receipt_identity()
        else {
            return Ok(None);
        };
        let (Some(frame), Some(source_uri)) = (self.frame.as_ref(), self.document_origin()) else {
            return Ok(None);
        };
        Self::validate_receipt_geometry(frame)?;
        let katana_frame_renderer_session_epoch = self
            .frame_renderer_session_epoch()
            .ok_or_else(|| "HTML frame has no accepted KatanA renderer epoch".to_owned())?;
        let viewport_width = frame.viewport.logical_width();
        let viewport_height = frame.viewport.logical_height();
        Ok(Some(HtmlReceiptObservation {
            source_uri: source_uri.to_owned(),
            katana_adapter_instance_id,
            katana_renderer_session_epoch,
            katana_frame_renderer_session_epoch,
            frame_generation: frame.generation,
            viewport_width,
            viewport_height,
            scroll_y: frame.scroll_y,
            content_height: frame.content_height,
            idle: self.is_idle(),
        }))
    }

    fn receipt_identity(&self) -> Option<(u64, u64)> {
        Some((
            self.katana_adapter_instance_id?,
            self.katana_renderer_session_epoch?,
        ))
    }

    fn validate_receipt_geometry(frame: &super::frame::BrowserFrame) -> Result<(), String> {
        if [
            frame.viewport.logical_width(),
            frame.viewport.logical_height(),
            frame.scroll_y,
            frame.content_height,
        ]
        .into_iter()
        .all(f32::is_finite)
        {
            return Ok(());
        }
        Err("HTML receipt observation contains non-finite geometry".to_owned())
    }
}
