use super::side_panels::PreviewSidePanels;

impl<'a> PreviewSidePanels<'a> {
    pub(super) fn apply_toc_click_scroll(&mut self, clicked_line: Option<usize>) {
        let Some(clicked) = clicked_line else {
            return;
        };
        self.app.state.scroll.scroll_to_line = None;
        self.app.state.scroll.toc_scroll_to_line = None;
        self.app.state.scroll.last_scroll_to_line = None;
        if self.app.state.active_view_mode() != crate::app_state::ViewMode::PreviewOnly {
            self.app.state.scroll.toc_scroll_to_line = Some(clicked);
        }
    }
}
