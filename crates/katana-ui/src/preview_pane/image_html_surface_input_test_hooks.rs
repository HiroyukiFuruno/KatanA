use super::HtmlBrowserSurface;
use katana_document_viewer::browser_session::HtmlBrowserInput;

impl HtmlBrowserSurface {
    pub(super) fn dispatch_input_burst_for_test(&mut self, count: u32) -> Result<(), String> {
        let viewport = self
            .frame
            .as_ref()
            .map(|frame| frame.viewport)
            .ok_or_else(|| "HTML browser frame is not ready".to_string())?;
        if self.adapter.is_none() {
            return Err("HTML browser adapter is not running".to_string());
        }
        for index in 0..count {
            let x = (index % viewport.width) as f32;
            let y = viewport.height as f32 / 2.0;
            self.dispatch(HtmlBrowserInput::PointerMove { x, y });
            self.ensure_input_dispatch_succeeded()?;
            self.dispatch(HtmlBrowserInput::Scroll {
                delta_x: 0.0,
                delta_y: 1.0,
            });
            self.ensure_input_dispatch_succeeded()?;
        }
        self.focused = true;
        self.dispatch(HtmlBrowserInput::Focus { focused: true });
        self.ensure_input_dispatch_succeeded()
    }

    fn ensure_input_dispatch_succeeded(&self) -> Result<(), String> {
        match self.error.as_ref() {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview_pane::image_html_surface::frame::BrowserFrame;

    #[test]
    fn burst_input_finishes_with_a_discrete_focus_barrier() -> Result<(), String> {
        let source = katana_document_viewer::browser_session::HtmlBrowserSource::new(
            "<p>html</p>",
            "https://example.test/index.html",
        )
        .map_err(|error| error.to_string())?;
        let viewport = katana_document_viewer::browser_session::HtmlBrowserViewport::new(2, 1, 1.0)
            .map_err(|error| error.to_string())?;
        let mut surface = HtmlBrowserSurface::start(source);
        assert!(surface.start_pending_session(viewport));
        surface.frame = Some(BrowserFrame::new(1, viewport, 0.0, 1.0, vec![0; 8]));

        surface.dispatch_input_burst_for_test(1)?;

        assert!(surface.focused);
        assert!(surface.error.is_none());
        Ok(())
    }
}
