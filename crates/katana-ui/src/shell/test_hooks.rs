mod document;
mod html;

use super::KatanaApp;
use crate::app_state::AppState;

impl KatanaApp {
    #[doc(hidden)]
    pub fn preview_geometry_for_test(&self) -> Option<(eframe::egui::Rect, f32, f32)> {
        let active_path = self.state.active_path()?;
        let pane = &self
            .tab_previews
            .iter()
            .find(|preview| preview.path == active_path)?
            .pane;
        Some((
            pane.markdown_viewport?,
            pane.markdown_scroll_offset,
            pane.content_top_y,
        ))
    }

    #[doc(hidden)]
    pub fn preview_anchors_for_test(
        &self,
    ) -> Option<Vec<(&'static str, std::ops::Range<usize>, eframe::egui::Rect)>> {
        let active_path = self.state.active_path()?;
        let pane = &self
            .tab_previews
            .iter()
            .find(|preview| preview.path == active_path)?
            .pane;
        /* WHY: Heading hit regions are synthesized and must stay distinct from real block bounds. */
        Some(
            pane.heading_anchors
                .iter()
                .map(|(lines, rect)| ("heading_hit", lines.clone(), *rect))
                .chain(
                    pane.block_anchors
                        .iter()
                        .map(|(lines, rect)| ("block_anchor", lines.clone(), *rect)),
                )
                .collect(),
        )
    }

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
    pub fn preview_resource_counts_for_test(&self) -> (usize, usize, usize, usize, usize, usize) {
        super::resource_snapshot::UiResourceSnapshot::capture(self)
            .0
            .preview_counts_tuple()
    }

    #[doc(hidden)]
    pub fn document_lifecycle_resources_for_test(
        &self,
    ) -> (usize, super::DocumentResourceSnapshotForTest) {
        super::resource_snapshot::UiResourceSnapshot::capture(self)
            .0
            .document_lifecycle_tuple()
    }

    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
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
