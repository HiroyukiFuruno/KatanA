use crate::app::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_close_other_documents(&mut self, idx: usize) {
        if idx >= self.state.document.open_documents.len() {
            return;
        }
        let target_doc_path = self.state.document.open_documents[idx].path.clone();
        let mut keep = Vec::new();
        let mut new_active_idx = None;
        let old_docs = std::mem::take(&mut self.state.document.open_documents);
        for doc in old_docs.into_iter() {
            let is_target = doc.path == target_doc_path;
            if is_target {
                new_active_idx = Some(keep.len());
                keep.push(doc);
            } else if doc.is_pinned {
                keep.push(doc);
            } else {
                self.state.push_recently_closed(doc.path, doc.is_pinned);
            }
        }
        self.state.document.open_documents = keep;
        if let Some(a_idx) = new_active_idx {
            self.state.document.active_doc_idx = Some(a_idx);
        }
        self.state.document.cleanup_empty_groups();
        self.save_workspace_state();
    }

    pub(super) fn handle_action_close_all_documents(&mut self) {
        let mut keep = Vec::new();
        let old_docs = std::mem::take(&mut self.state.document.open_documents);
        for doc in old_docs.into_iter() {
            if doc.is_pinned {
                keep.push(doc);
            } else {
                self.state.push_recently_closed(doc.path, doc.is_pinned);
            }
        }
        self.state.document.open_documents = keep;
        if self.state.document.open_documents.is_empty() {
            self.state.document.active_doc_idx = None;
        } else if self.state.document.active_doc_idx.is_some() {
            self.state.document.active_doc_idx = Some(0);
        }
        self.state.document.cleanup_empty_groups();
        self.save_workspace_state();
        self.cleanup_closed_tab_previews();
    }

    pub(super) fn handle_action_close_to_right(&mut self, idx: usize) {
        let mut keep = Vec::new();
        let active_path = self.state.active_document().map(|d| d.path.clone());
        let old_docs = std::mem::take(&mut self.state.document.open_documents);
        for (i, doc) in old_docs.into_iter().enumerate() {
            if i <= idx || doc.is_pinned {
                keep.push(doc);
            } else {
                self.state.push_recently_closed(doc.path, doc.is_pinned);
            }
        }
        self.state.document.open_documents = keep;
        if let Some(p) = active_path {
            let new_idx = self
                .state
                .document
                .open_documents
                .iter()
                .position(|d| d.path == p);
            self.state.document.active_doc_idx = new_idx.or(Some(
                idx.min(self.state.document.open_documents.len().saturating_sub(1)),
            ));
        }
        self.state.document.cleanup_empty_groups();
        self.save_workspace_state();
        self.cleanup_closed_tab_previews();
    }

    pub(super) fn handle_action_close_to_left(&mut self, idx: usize) {
        let mut keep = Vec::new();
        let active_path = self.state.active_document().map(|d| d.path.clone());
        let old_docs = std::mem::take(&mut self.state.document.open_documents);
        for (i, doc) in old_docs.into_iter().enumerate() {
            if i >= idx || doc.is_pinned {
                keep.push(doc);
            } else {
                self.state.push_recently_closed(doc.path, doc.is_pinned);
            }
        }
        self.state.document.open_documents = keep;
        if let Some(p) = active_path {
            let new_idx = self
                .state
                .document
                .open_documents
                .iter()
                .position(|d| d.path == p);
            self.state.document.active_doc_idx = new_idx.or(Some(0));
        }
        self.state.document.cleanup_empty_groups();
        self.save_workspace_state();
        self.cleanup_closed_tab_previews();
    }

    pub(super) fn handle_action_toggle_pin(&mut self, idx: usize) {
        if idx < self.state.document.open_documents.len() {
            let active_path = self.state.active_document().map(|d| d.path.clone());
            let doc = &mut self.state.document.open_documents[idx];
            let is_now_pinned = !doc.is_pinned;
            doc.is_pinned = is_now_pinned;
            let doc_path = doc.path.clone();
            if is_now_pinned {
                let doc_str = doc_path.to_string_lossy().to_string();
                for g in &mut self.state.document.tab_groups {
                    g.members.retain(|m| m != &doc_str);
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
        self.save_workspace_state();
    }

    pub(super) fn handle_action_restore_closed_document(&mut self) {
        let Some((path, is_pinned)) = self.state.document.recently_closed_tabs.pop_back() else {
            return;
        };
        self.handle_select_document(path.clone(), true);
        if let Some(doc) = self
            .state
            .document
            .open_documents
            .iter_mut()
            .find(|d| d.path == path)
        {
            doc.is_pinned = is_pinned;
        }
        let active_path = self.state.active_document().map(|d| d.path.clone());
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
        self.save_workspace_state();
    }
}
