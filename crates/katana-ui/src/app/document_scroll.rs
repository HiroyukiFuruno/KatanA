use crate::shell::KatanaApp;
use std::path::{Path, PathBuf};

pub(crate) trait DocumentScrollOps {
    fn reset_scroll_for_new_active_path(&mut self, previous: &Option<PathBuf>, next: &Path);
}

impl DocumentScrollOps for KatanaApp {
    fn reset_scroll_for_new_active_path(&mut self, previous: &Option<PathBuf>, next: &Path) {
        if previous.as_deref() != Some(next) {
            self.state.scroll.reset_for_document_change();
        }
    }
}
