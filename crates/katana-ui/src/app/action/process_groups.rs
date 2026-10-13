use crate::app::*;
use crate::shell::*;

impl KatanaApp {
    fn prepare_members_for_group(&mut self, members: &[std::path::PathBuf]) -> Vec<String> {
        let member_strs: Vec<String> = members
            .iter()
            .map(|m| m.to_string_lossy().to_string())
            .collect();
        for g in &mut self.state.document.tab_groups {
            g.members.retain(|m| !member_strs.contains(m));
        }
        for m in members {
            if let Some(doc) = self
                .state
                .document
                .open_documents
                .iter_mut()
                .find(|d| d.path == *m)
            {
                doc.is_pinned = false;
            }
        }
        member_strs
    }

    pub(super) fn handle_action_create_tab_group(
        &mut self,
        name: String,
        color_hex: String,
        initial_members: Vec<std::path::PathBuf>,
    ) {
        let id = format!("group_{}", chrono::Utc::now().timestamp_micros());
        let member_strs = self.prepare_members_for_group(&initial_members);
        self.state
            .document
            .tab_groups
            .retain(|g| !g.members.is_empty());
        self.state
            .document
            .tab_groups
            .push(crate::state::document::TabGroup {
                id: id.clone(),
                name,
                color_hex,
                collapsed: false,
                members: member_strs,
            });
        self.state.layout.inline_rename_group = Some(id);
        self.save_workspace_state();
        let mut to_open = Vec::new();
        for m in initial_members {
            if !self
                .state
                .document
                .open_documents
                .iter()
                .any(|d| d.path == m)
            {
                to_open.push(m);
            }
        }
        if !to_open.is_empty() {
            self.handle_action_open_multiple(to_open);
        }
    }

    pub(super) fn handle_action_add_tabs_to_group(
        &mut self,
        group_id: String,
        members: Vec<std::path::PathBuf>,
    ) {
        let member_strs = self.prepare_members_for_group(&members);
        let found = if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter_mut()
            .find(|g| g.id == group_id && !g.is_demo())
        {
            g.members.extend(member_strs);
            true
        } else {
            false
        };
        if found {
            let target_id = group_id.clone();
            self.state
                .document
                .tab_groups
                .retain(|g| g.id == target_id || !g.members.is_empty());
        }
        self.save_workspace_state();
        let mut to_open = Vec::new();
        for m in members {
            if !self
                .state
                .document
                .open_documents
                .iter()
                .any(|d| d.path == m)
            {
                to_open.push(m);
            }
        }
        if !to_open.is_empty() {
            self.handle_action_open_multiple(to_open);
        }
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
}
