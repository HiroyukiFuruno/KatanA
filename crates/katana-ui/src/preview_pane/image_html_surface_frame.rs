use super::HtmlBrowserSurface;
use eframe::egui;
use katana_document_viewer::browser_session::HtmlBrowserViewport;

pub(super) struct BrowserFrame {
    pub(super) generation: u64,
    renderer_session_epoch: Option<u64>,
    pub(super) viewport: HtmlBrowserViewport,
    pub(super) scroll_y: f32,
    pub(super) content_height: f32,
    pub(super) pixels: Vec<u8>,
}

impl BrowserFrame {
    pub(super) fn new(
        generation: u64,
        viewport: HtmlBrowserViewport,
        scroll_y: f32,
        content_height: f32,
        pixels: Vec<u8>,
    ) -> Self {
        Self {
            generation,
            renderer_session_epoch: None,
            viewport,
            scroll_y,
            content_height,
            pixels,
        }
    }

    pub(super) fn for_renderer_session(
        generation: u64,
        viewport: HtmlBrowserViewport,
        scroll_y: f32,
        content_height: f32,
        pixels: Vec<u8>,
        epoch: Option<u64>,
    ) -> Self {
        /* WHY: frame受理時のKatanA epochを保持し、receipt時に現在値から推定しない。 */
        let mut accepted = Self::new(generation, viewport, scroll_y, content_height, pixels);
        accepted.renderer_session_epoch = epoch;
        accepted
    }
}

impl HtmlBrowserSurface {
    pub(super) fn frame_renderer_session_epoch(&self) -> Option<u64> {
        self.frame.as_ref()?.renderer_session_epoch
    }

    pub(super) fn update_texture(&mut self, ctx: &egui::Context) {
        let Some(frame) = &self.frame else {
            return;
        };
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [
                frame.viewport.width as usize,
                frame.viewport.height as usize,
            ],
            &frame.pixels,
        );
        if let Some(texture) = &mut self.texture {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            self.texture = Some(ctx.load_texture(
                format!("html_browser_frame_{}", frame.generation),
                image,
                egui::TextureOptions::LINEAR,
            ));
        }
    }
}
