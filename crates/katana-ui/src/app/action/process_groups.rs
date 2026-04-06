use crate::app::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_create_tab_group(
        &mut self,
        name: String,
        color_hex: String,
        initial_member: std::path::PathBuf,
    ) {
        let id = format!("group_{}", chrono::Utc::now().timestamp_micros());
        let member_str = initial_member.to_string_lossy().to_string();
        for g in &mut self.state.document.tab_groups {
            g.members.retain(|m| m != &member_str);
        }
        if let Some(doc) = self
            .state
            .document
            .open_documents
            .iter_mut()
            .find(|d| d.path == initial_member)
        {
            doc.is_pinned = false;
        }
        let members = vec![member_str];
        self.state
            .document
            .tab_groups
            .push(crate::state::document::TabGroup {
                id: id.clone(),
                name,
                color_hex,
                collapsed: false,
                members,
            });
        self.state.layout.inline_rename_group = Some(id);
        self.save_workspace_state();
    }

    pub(super) fn handle_action_add_tab_to_group(
        &mut self,
        group_id: String,
        member: std::path::PathBuf,
    ) {
        let member_str = member.to_string_lossy().to_string();
        for g in &mut self.state.document.tab_groups {
            g.members.retain(|m| m != &member_str);
        }
        if let Some(doc) = self
            .state
            .document
            .open_documents
            .iter_mut()
            .find(|d| d.path == member)
        {
            doc.is_pinned = false;
        }
        if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter_mut()
            .find(|g| g.id == group_id)
        {
            g.members.push(member_str);
        }
        self.save_workspace_state();
    }

    pub(super) fn handle_action_remove_tab_from_group(&mut self, member: std::path::PathBuf) {
        let member_str = member.to_string_lossy().to_string();
        for g in &mut self.state.document.tab_groups {
            g.members.retain(|m| m != &member_str);
        }
        self.state
            .document
            .tab_groups
            .retain(|g| !g.members.is_empty());
        self.save_workspace_state();
    }

    pub(super) fn handle_action_rename_tab_group(&mut self, group_id: String, new_name: String) {
        if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter_mut()
            .find(|g| g.id == group_id)
        {
            g.name = new_name;
        }
        self.save_workspace_state();
    }

    pub(super) fn handle_action_recolor_tab_group(&mut self, group_id: String, new_color: String) {
        if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter_mut()
            .find(|g| g.id == group_id)
        {
            g.color_hex = new_color;
        }
        self.save_workspace_state();
    }

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
