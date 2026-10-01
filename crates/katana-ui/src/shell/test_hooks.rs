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
        let previews = self.tab_previews.len();
        let mut html_surfaces = 0;
        let mut document_surfaces = 0;
        let mut frames = 0;
        let mut textures = 0;
        let mut cache_entries = 0;
        for preview in &self.tab_previews {
            let (preview_html_surfaces, html_frames, html_textures) =
                preview.pane.html_browser_resource_counts_for_test();
            let (preview_document_surfaces, document_frames, document_textures) =
                preview.pane.document_resource_counts_for_test();
            html_surfaces += preview_html_surfaces;
            document_surfaces += preview_document_surfaces;
            frames += html_frames + document_frames;
            textures += html_textures + document_textures;
            cache_entries += preview.pane.sections.len()
                + preview.pane.image_cache.len()
                + preview.pane.viewer_states.len();
        }
        (
            previews,
            html_surfaces,
            document_surfaces,
            frames,
            textures,
            cache_entries,
        )
    }

    #[doc(hidden)]
    pub fn document_lifecycle_resources_for_test(
        &self,
    ) -> (usize, super::DocumentResourceSnapshotForTest) {
        (
            crate::preview_pane::DocumentWorkerLifecycle::live_count(),
            katana_document_viewer::DocumentSession::resource_snapshot(),
        )
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
