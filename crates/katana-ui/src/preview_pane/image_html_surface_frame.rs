use super::HtmlBrowserSurface;
use eframe::egui;
use katana_document_viewer::browser_session::HtmlBrowserViewport;

pub(super) struct BrowserFrame {
    pub(super) generation: u64,
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
            viewport,
            scroll_y,
            content_height,
            pixels,
        }
    }
}

impl HtmlBrowserSurface {
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
