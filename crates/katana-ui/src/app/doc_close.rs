use crate::app::*;
use crate::shell::*;

pub(super) trait DocCloseOps {
    fn force_close_document(&mut self, idx: usize);
}

impl DocCloseOps for KatanaApp {
    fn force_close_document(&mut self, idx: usize) {
        if idx < self.state.document.open_documents.len() {
            let closed_doc = self.state.document.open_documents.remove(idx);
            let path_str = closed_doc.path.to_string_lossy().to_string();
            for g in &mut self.state.document.tab_groups {
                g.members.retain(|m| m != &path_str);
            }
            self.state
                .document
                .tab_groups
                .retain(|g| !g.members.is_empty());
            self.state
                .push_recently_closed(closed_doc.path.clone(), closed_doc.is_pinned);
            self.state.document.active_doc_idx = if self.state.document.open_documents.is_empty() {
                None
            } else {
                Some(
                    self.state
                        .document
                        .active_doc_idx
                        .unwrap_or(0)
                        .saturating_sub(if self.state.document.active_doc_idx == Some(idx) {
                            1
                        } else {
                            0
                        })
                        .min(self.state.document.open_documents.len().saturating_sub(1)),
                )
            };
        }
        self.save_workspace_state();
        self.cleanup_closed_tab_previews();
    }
}
