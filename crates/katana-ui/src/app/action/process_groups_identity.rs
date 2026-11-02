use crate::app::workspace::WorkspaceOps;
use crate::shell::KatanaApp;

impl KatanaApp {
    pub(super) fn can_open_demo_group(&mut self) -> bool {
        let groups = &self.state.document.tab_groups;
        if groups.iter().any(|group| group.id == "demo") {
            return true;
        }
        if crate::state::group_identity::GroupIdentity::conflicts(groups, None, "Demo", "#808080") {
            self.show_group_identity_error();
            return false;
        }
        true
    }

    pub(super) fn handle_action_rename_tab_group(&mut self, group_id: String, new_name: String) {
        if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter()
            .find(|g| g.id == group_id)
        {
            self.update_tab_group_identity(&group_id, new_name, g.color_hex.clone());
        }
    }

    pub(super) fn handle_action_recolor_tab_group(&mut self, group_id: String, new_color: String) {
        if let Some(g) = self
            .state
            .document
            .tab_groups
            .iter()
            .find(|g| g.id == group_id)
        {
            self.update_tab_group_identity(&group_id, g.name.clone(), new_color);
        }
    }

    pub(super) fn update_tab_group_identity(
        &mut self,
        id: &str,
        name: String,
        color: String,
    ) -> bool {
        let groups = &self.state.document.tab_groups;
        let Some(index) = groups.iter().position(|group| group.id == id) else {
            return false;
        };
        let current = &groups[index];
        if crate::state::group_identity::GroupIdentity::same_identity(
            &current.name,
            &current.color_hex,
            &name,
            &color,
        ) {
            return true;
        }
        if crate::state::group_identity::GroupIdentity::conflicts(groups, Some(id), &name, &color) {
            self.show_group_identity_error();
            return false;
        }
        let group = &mut self.state.document.tab_groups[index];
        group.name = name;
        group.color_hex = color;
        self.save_workspace_state();
        true
    }
    pub(super) fn show_group_identity_error(&mut self) {
        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::get()
                .tab
                .group_identity_duplicate
                .clone(),
            crate::app_state::StatusType::Error,
        ));
    }
}
