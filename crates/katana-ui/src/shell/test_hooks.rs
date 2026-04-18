use super::KatanaApp;
use crate::app_state::AppState;

impl KatanaApp {
    #[doc(hidden)]
    pub fn disable_update_check_for_test(&mut self) {
        self.update_rx = None;
    }

    #[doc(hidden)]
    pub fn disable_changelog_popup_for_test(&mut self) {
        self.needs_changelog_display = false;
        self.show_update_dialog = false;
    }

    #[doc(hidden)]
    pub fn enable_changelog_popup_for_test(&mut self) {
        self.needs_changelog_display = true;
    }

    #[doc(hidden)]
    pub fn app_state_for_test(&self) -> &AppState {
        &self.state
    }

    #[doc(hidden)]
    pub fn pending_action_for_test(&self) -> &crate::app_state::AppAction {
        &self.pending_action
    }

    #[doc(hidden)]
    pub fn needs_changelog_for_test(&self) -> bool {
        self.needs_changelog_display
    }

    #[doc(hidden)]
    pub fn explorer_rx_for_test(&self) -> bool {
        self.explorer_rx.is_some()
    }

    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    #[doc(hidden)]
    pub fn set_changelog_sections_for_test(
        &mut self,
        sections: Vec<crate::changelog::ChangelogSection>,
    ) {
        self.changelog_sections = sections;
    }

    #[doc(hidden)]
    pub fn clear_changelog_rx_for_test(&mut self) {
        self.changelog_rx = None;
    }
}
