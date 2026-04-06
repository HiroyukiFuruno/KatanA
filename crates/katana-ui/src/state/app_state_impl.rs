use crate::app_state::AppState;
use crate::state::document::SplitViewState;
use crate::state::document::{DocumentState, TabSplitState, TabViewMode, ViewMode};

impl AppState {
    pub fn set_active_view_mode(&mut self, mode: ViewMode) {
        if let Some(doc) = self.active_document() {
            let path = doc.path.clone();
            if let Some(t) = self
                .document
                .tab_view_modes
                .iter_mut()
                .find(|t| t.path == path)
            {
                t.mode = mode;
            } else {
                self.document
                    .tab_view_modes
                    .push(TabViewMode { path, mode });
            }
        }
    }

    pub(crate) fn split_defaults(&self) -> SplitViewState {
        SplitViewState {
            direction: self.config.settings.settings().layout.split_direction,
            order: self.config.settings.settings().layout.pane_order,
        }
    }

    pub fn initialize_tab_split_state(&mut self, path: impl Into<std::path::PathBuf>) {
        let p = path.into();
        if !self.document.tab_split_states.iter().any(|t| t.path == p) {
            let defaults = self.split_defaults();
            self.document.tab_split_states.push(TabSplitState {
                path: p,
                state: defaults,
            });
        }
    }

    pub fn ensure_active_split_state(&mut self) {
        let Some(path) = self.active_path() else {
            return;
        };
        self.initialize_tab_split_state(path);
    }

    pub fn active_split_direction(&self) -> katana_platform::SplitDirection {
        self.active_document()
            .and_then(|doc| {
                self.document
                    .tab_split_states
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.state.direction)
            })
            .unwrap_or_else(|| self.split_defaults().direction)
    }

    pub fn active_pane_order(&self) -> katana_platform::PaneOrder {
        self.active_document()
            .and_then(|doc| {
                self.document
                    .tab_split_states
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.state.order)
            })
            .unwrap_or_else(|| self.split_defaults().order)
    }

    pub fn set_active_split_direction(&mut self, dir: katana_platform::SplitDirection) {
        let Some(path) = self.active_path() else {
            return;
        };
        if let Some(t) = self
            .document
            .tab_split_states
            .iter_mut()
            .find(|t| t.path == path)
        {
            t.state.direction = dir;
        } else {
            let mut defaults = self.split_defaults();
            defaults.direction = dir;
            self.document.tab_split_states.push(TabSplitState {
                path,
                state: defaults,
            });
        }
    }

    pub fn set_active_pane_order(&mut self, order: katana_platform::PaneOrder) {
        let Some(path) = self.active_path() else {
            return;
        };
        if let Some(t) = self
            .document
            .tab_split_states
            .iter_mut()
            .find(|t| t.path == path)
        {
            t.state.order = order;
        } else {
            let mut defaults = self.split_defaults();
            defaults.order = order;
            self.document.tab_split_states.push(TabSplitState {
                path,
                state: defaults,
            });
        }
    }

    pub fn push_recently_closed(&mut self, path: std::path::PathBuf, is_pinned: bool) {
        if self.document.recently_closed_tabs.len() >= DocumentState::MAX_RECENTLY_CLOSED_TABS {
            self.document.recently_closed_tabs.pop_front();
        }
        self.document
            .recently_closed_tabs
            .push_back((path, is_pinned));
    }
}
