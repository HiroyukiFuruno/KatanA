use super::KatanaApp;

impl KatanaApp {
    #[doc(hidden)]
    pub fn document_frame_for_test(
        &self,
    ) -> Option<(std::path::PathBuf, String, usize, usize, String)> {
        let active_path = self.state.active_path()?;
        let (format, active_index, item_count, node_kind) = self
            .tab_previews
            .iter()
            .find(|preview| preview.path == active_path)?
            .pane
            .document_frame_state_for_test()?;
        Some((active_path, format, active_index, item_count, node_kind))
    }

    #[doc(hidden)]
    pub fn document_failure_for_test(&self) -> Option<String> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)?
            .pane
            .document_failure_for_test()
    }

    #[doc(hidden)]
    pub fn document_is_idle_for_test(&self) -> Option<bool> {
        let active_path = self.state.active_path()?;
        self.tab_previews
            .iter()
            .find(|preview| preview.path == active_path)?
            .pane
            .document_is_idle_for_test()
    }

    #[doc(hidden)]
    pub fn document_next_for_test(&mut self) -> Result<(), String> {
        let active_path = self
            .state
            .active_path()
            .ok_or_else(|| "active document is missing".to_owned())?;
        let preview = self
            .tab_previews
            .iter_mut()
            .find(|preview| preview.path == active_path)
            .ok_or_else(|| "active document preview is missing".to_owned())?;
        if preview.pane.document_next_for_test() {
            Ok(())
        } else {
            Err("active document has no next item".to_owned())
        }
    }
}
