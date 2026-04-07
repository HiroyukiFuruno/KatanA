use super::KatanaApp;
use crate::app_state::AppState;

impl KatanaApp {
    #[doc(hidden)]
    pub fn disable_update_check_for_test(&mut self) {
        self.update_rx = None;
    }

    #[doc(hidden)]
    pub fn app_state_for_test(&self) -> &AppState {
        &self.state
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
