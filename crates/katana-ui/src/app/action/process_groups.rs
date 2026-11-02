use crate::app::*;
use crate::shell::*;

#[cfg(test)]
#[path = "process_groups_identity_tests.rs"]
mod identity_tests;

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
        let direct = !name.trim().is_empty();
        self.state.layout.group_draft = Some(crate::state::layout::GroupDraft {
            name,
            color_hex,
            members: initial_members,
        });
        if direct {
            self.commit_tab_group_draft();
        }
    }

    pub(crate) fn commit_tab_group_draft(&mut self) {
        let Some(draft) = self.state.layout.group_draft.as_ref() else {
            return;
        };
        if crate::state::group_identity::GroupIdentity::conflicts(
            &self.state.document.tab_groups,
            None,
            &draft.name,
            &draft.color_hex,
        ) {
            self.show_group_identity_error();
            return;
        }
        let Some(draft) = self.state.layout.group_draft.take() else {
            return;
        };
        self.create_valid_tab_group(draft.name, draft.color_hex, draft.members);
    }

    fn create_valid_tab_group(
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
        self.save_workspace_state();
        self.open_missing_group_members(initial_members);
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
        self.open_missing_group_members(members);
    }

    fn open_missing_group_members(&mut self, members: Vec<std::path::PathBuf>) {
        let to_open = members
            .into_iter()
            .filter(|path| {
                !self
                    .state
                    .document
                    .open_documents
                    .iter()
                    .any(|doc| doc.path == *path)
            })
            .collect::<Vec<_>>();
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
        self.clear_removed_group_rename();
        self.save_workspace_state();
    }

    fn clear_removed_group_rename(&mut self) {
        if self
            .state
            .layout
            .inline_rename_group
            .as_ref()
            .is_some_and(|id| {
                !self
                    .state
                    .document
                    .tab_groups
                    .iter()
                    .any(|group| &group.id == id)
            })
        {
            self.state.layout.inline_rename_group = None;
        }
    }
}
