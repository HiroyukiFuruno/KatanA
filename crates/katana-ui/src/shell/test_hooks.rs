use super::KatanaApp;
use crate::app_state::AppState;

const RGB_COMPONENT_COUNT: usize = 3;

impl KatanaApp {
    #[doc(hidden)]
    pub fn disable_update_check_for_test(&mut self) {
        self.update_rx = None;
    }

    #[doc(hidden)]
    pub fn disable_changelog_popup_for_test(&mut self) {
        self.needs_changelog_display = false;
        self.show_update_dialog = false;
    }

    #[doc(hidden)]
    pub fn enable_changelog_popup_for_test(&mut self) {
        self.needs_changelog_display = true;
    }

    #[doc(hidden)]
    pub fn app_state_for_test(&self) -> &AppState {
        &self.state
    }

    #[doc(hidden)]
    pub fn pending_action_for_test(&self) -> &crate::app_state::AppAction {
        &self.pending_action
    }

    #[doc(hidden)]
    pub fn needs_changelog_for_test(&self) -> bool {
        self.needs_changelog_display
    }

    #[doc(hidden)]
    pub fn explorer_rx_for_test(&self) -> bool {
        self.explorer_rx.is_some()
    }

    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    #[doc(hidden)]
    pub fn html_browser_origin_for_test(&self) -> Option<String> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.html_browser_origin())
    }

    #[doc(hidden)]
    pub fn html_browser_navigation_history_for_test(&self) -> Option<Vec<String>> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.html_browser_navigation_history())
    }

    #[doc(hidden)]
    pub fn html_browser_frame_matching_rgb_pixels_for_test(
        &self,
        expected: [u8; RGB_COMPONENT_COUNT],
    ) -> Option<u64> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| {
                preview
                    .pane
                    .html_browser_frame_matching_rgb_pixels(expected)
            })
    }

    #[doc(hidden)]
    pub fn html_browser_frame_viewport_for_test(&self) -> Option<(f32, f32)> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.html_browser_frame_viewport())
    }

    #[doc(hidden)]
    pub fn html_browser_frame_scroll_metrics_for_test(&self) -> Option<(f32, f32)> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.html_browser_frame_scroll_metrics())
    }

    #[doc(hidden)]
    pub fn html_browser_display_rect_for_test(&self) -> Option<eframe::egui::Rect> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)
            .and_then(|preview| preview.pane.html_browser_display_rect())
    }

    #[doc(hidden)]
    pub fn set_changelog_sections_for_test(
        &mut self,
        sections: Vec<crate::changelog::ChangelogSection>,
    ) {
        self.changelog_sections = sections;
    }

    #[doc(hidden)]
    pub fn clear_changelog_rx_for_test(&mut self) {
        self.changelog_rx = None;
    }
}
