use crate::shell::KatanaApp;
use std::path::Path;

impl KatanaApp {
    pub(super) fn migrate_binary_url_target(&mut self, previous: Option<&Path>, next: &Path) {
        let Some(previous) = previous.filter(|previous| *previous != next) else {
            return;
        };
        if self
            .state
            .document
            .open_documents
            .iter()
            .any(|document| document.path == next)
        {
            return;
        }
        let Some(index) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|document| document.path == previous && !document.is_dirty)
        else {
            return;
        };
        /* WHY: 未保存の元HTMLは別タブとして保持し、移行可能なタブだけ所有先を変える。 */
        let pinned = self.state.document.open_documents[index].is_pinned;
        let mut document = katana_core::document::Document::new(next, String::new());
        document.is_pinned = pinned;
        document.is_reference = true;
        self.state.document.open_documents[index] = document;
        self.state.document.replace_path_references(previous, next);
        self.tab_previews.retain(|preview| preview.path != previous);
        self.state
            .url_tab
            .tabs
            .retain(|tab| tab.document_path != previous);
        self.state.url_tab.active_tab = None;
        self.state
            .url_tab
            .document_tabs
            .retain(|tab| tab.document_path != previous);
    }
}
