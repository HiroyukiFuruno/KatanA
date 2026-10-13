use super::types::*;

impl<'a> TocPanel<'a> {
    pub fn new(
        preview: &'a mut crate::preview_pane::PreviewPane,
        state: &'a mut crate::app_state::AppState,
    ) -> Self {
        Self { preview, state }
    }
}
