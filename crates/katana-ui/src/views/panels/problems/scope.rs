use crate::app_state::AppState;
use crate::state::diagnostics::ProblemsScope;
use eframe::egui;

pub(super) struct ProblemScopeOps;

impl ProblemScopeOps {
    pub(super) fn render_toggle(ui: &mut egui::Ui, state: &mut AppState) {
        const LEFT_MARGIN: f32 = 8.0;
        const SEGMENT_WIDTH: f32 = 112.0;

        let messages = &crate::i18n::I18nOps::get().status;
        let open_tabs_label = messages.problems_scope_open_tabs.clone();
        let active_tab_label = messages.problems_scope_active_tab.clone();
        let choices = vec![open_tabs_label.as_str(), active_tab_label.as_str()];
        let mut selected =
            Self::selected_label(state.diagnostics.scope, &open_tabs_label, &active_tab_label);

        ui.add_space(LEFT_MARGIN);
        let response = ui.add(
            crate::widgets::SegmentedStringToggle::new(
                "problems-scope-toggle",
                &choices,
                &mut selected,
            )
            .segment_width(SEGMENT_WIDTH),
        );

        if response.changed() {
            state.diagnostics.scope = Self::scope_from_label(&selected, &active_tab_label);
        }
    }

    pub(super) fn visible_paths(state: &AppState) -> Vec<std::path::PathBuf> {
        match state.diagnostics.scope {
            ProblemsScope::OpenTabs => state
                .document
                .open_documents
                .iter()
                .map(|doc| doc.path.clone())
                .collect(),
            ProblemsScope::ActiveTab => state
                .active_document()
                .map(|doc| vec![doc.path.clone()])
                .unwrap_or_default(),
        }
    }

    fn selected_label(
        scope: ProblemsScope,
        open_tabs_label: &str,
        active_tab_label: &str,
    ) -> String {
        match scope {
            ProblemsScope::OpenTabs => open_tabs_label.to_string(),
            ProblemsScope::ActiveTab => active_tab_label.to_string(),
        }
    }

    fn scope_from_label(selected: &str, active_tab_label: &str) -> ProblemsScope {
        if selected == active_tab_label {
            ProblemsScope::ActiveTab
        } else {
            ProblemsScope::OpenTabs
        }
    }
}
