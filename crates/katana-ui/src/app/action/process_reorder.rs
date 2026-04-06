use crate::app::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_reorder_document(
        &mut self,
        from: usize,
        to: usize,
        new_group_id: Option<Option<String>>,
    ) {
        let len = self.state.document.open_documents.len();
        if from < len && to <= len && from != to {
            self.reorder_documents_different_pos(from, to, new_group_id);
        } else if from == to {
            self.reorder_documents_same_pos(from, new_group_id);
        }
        self.save_workspace_state();
    }

    fn reorder_documents_different_pos(
        &mut self,
        from: usize,
        to: usize,
        new_group_id: Option<Option<String>>,
    ) {
        let active_path = self.state.active_document().map(|d| d.path.clone());
        let doc = self.state.document.open_documents.remove(from);
        let is_doc_pinned = doc.is_pinned;
        if let Some(group_option) = new_group_id {
            let doc_str = doc.path.to_string_lossy().to_string();
            for g in &mut self.state.document.tab_groups {
                g.members.retain(|m| m != &doc_str);
            }
            if let Some(target_g_id) = group_option
                && !is_doc_pinned
                && let Some(g) = self
                    .state
                    .document
                    .tab_groups
                    .iter_mut()
                    .find(|g| g.id == target_g_id)
            {
                g.members.push(doc_str);
            }
        }
        let actual_to = if to > from { to - 1 } else { to };
        self.state.document.open_documents.insert(actual_to, doc);
        self.state
            .document
            .open_documents
            .sort_by_key(|d| !d.is_pinned);
        if let Some(path) = active_path
            && let Some(new_idx) = self
                .state
                .document
                .open_documents
                .iter()
                .position(|d| d.path == path)
        {
            self.state.document.active_doc_idx = Some(new_idx);
        }
    }

    fn reorder_documents_same_pos(&mut self, from: usize, new_group_id: Option<Option<String>>) {
        let active_path = self.state.active_document().map(|d| d.path.clone());
        let is_doc_pinned = self.state.document.open_documents[from].is_pinned;
        if let Some(group_option) = new_group_id {
            let doc_str = self.state.document.open_documents[from]
                .path
                .to_string_lossy()
                .to_string();
            for g in &mut self.state.document.tab_groups {
                g.members.retain(|m| m != &doc_str);
            }
            if let Some(target_g_id) = group_option
                && !is_doc_pinned
                && let Some(g) = self
                    .state
                    .document
                    .tab_groups
                    .iter_mut()
                    .find(|g| g.id == target_g_id)
            {
                g.members.push(doc_str);
            }
        }
        self.state
            .document
            .open_documents
            .sort_by_key(|d| !d.is_pinned);
        if let Some(path) = active_path
            && let Some(new_idx) = self
                .state
                .document
                .open_documents
                .iter()
                .position(|d| d.path == path)
        {
            self.state.document.active_doc_idx = Some(new_idx);
        }
    }
}
