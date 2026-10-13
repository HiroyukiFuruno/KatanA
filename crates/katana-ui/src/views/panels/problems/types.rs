use crate::app_state::AppState;

pub struct ProblemsPanel<'a> {
    pub(crate) state: &'a mut AppState,
    pub(crate) pending_action: &'a mut crate::app_state::AppAction,
}
