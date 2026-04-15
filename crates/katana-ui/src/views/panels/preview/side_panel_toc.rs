use super::side_panels::PreviewSidePanels;
use eframe::egui;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn render_toc(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.layout.show_toc {
            return;
        }

        let doc = match self.app.state.active_document() {
            Some(d) => d,
            None => return,
        };

        if let Some(preview) = self
            .app
            .tab_previews
            .iter_mut()
            .find(|p| p.path == doc.path)
        {
            let (clicked_line, active_index, _panel_rect) =
                crate::views::panels::toc::TocPanel::new(&mut preview.pane, &mut self.app.state)
                    .show(ui);

            if let Some(clicked) = clicked_line {
                self.app.state.scroll.scroll_to_line = Some(clicked);
                self.app.state.scroll.last_scroll_to_line = None;
            }
            self.app.state.active_toc_index = active_index;
        }
    }
}
