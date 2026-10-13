use crate::app::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_close_tab_group(&mut self, group_id: String) {
        let members_to_close: Vec<String> = self
            .state
            .document
            .tab_groups
            .iter()
            .find(|g| g.id == group_id)
            .map(|g| g.members.clone())
            .unwrap_or_default();

        if !members_to_close.is_empty() {
            let active_path = self.state.active_document().map(|d| d.path.clone());
            let old_docs = std::mem::take(&mut self.state.document.open_documents);
            let mut keep = Vec::new();
            for doc in old_docs.into_iter() {
                if members_to_close.contains(&doc.path.to_string_lossy().to_string()) {
                    self.state.push_recently_closed(doc.path, doc.is_pinned);
                } else {
                    keep.push(doc);
                }
            }
            self.state.document.open_documents = keep;
            let docs_len = self.state.document.open_documents.len();
            if let Some(p) = active_path {
                let new_idx = self
                    .state
                    .document
                    .open_documents
                    .iter()
                    .position(|d| d.path == p);
                self.state.document.active_doc_idx = if docs_len > 0 {
                    new_idx.or(Some(docs_len - 1))
                } else {
                    None
                };
            } else {
                self.state.document.active_doc_idx = if docs_len > 0 {
                    Some(docs_len - 1)
                } else {
                    None
                };
            }
        }
        self.state.document.tab_groups.retain(|g| g.id != group_id);
        self.save_workspace_state();
    }

    pub(super) fn handle_action_toggle_collapse_tab_group(&mut self, group_id: String) {
        let mut collapsed = false;
        let mut group_members = Vec::new();
        if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter_mut()
            .find(|g| g.id == group_id)
        {
            g.collapsed = !g.collapsed;
            collapsed = g.collapsed;
            group_members = g.members.clone();
        }
        if collapsed {
            self.apply_collapse_active_doc(&group_members);
        }
        self.save_workspace_state();
    }

    fn apply_collapse_active_doc(&mut self, group_members: &[String]) {
        let active_idx = self.state.document.active_doc_idx.unwrap_or(0);
        let Some(active_doc) = self.state.document.open_documents.get(active_idx) else {
            return;
        };
        let path_str = active_doc.path.to_string_lossy().to_string();
        if !group_members.contains(&path_str) {
            return;
        }
        let new_idx = self
            .state
            .document
            .open_documents
            .iter()
            .position(|d| !group_members.contains(&d.path.to_string_lossy().to_string()));
        self.state.document.active_doc_idx = new_idx;
    }
}
