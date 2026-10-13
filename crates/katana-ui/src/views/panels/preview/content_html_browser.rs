use crate::{app_state::AppAction, preview_pane::PreviewPane};
use eframe::egui;

pub(super) fn show_html_browser_content(
    preview: &mut PreviewPane,
    ui: &mut egui::Ui,
    active_editor_line: Option<usize>,
    search_query: Option<String>,
    doc_search_active_index: Option<usize>,
    action: &mut AppAction,
) {
    let actions = preview.show_content(
        ui,
        active_editor_line,
        None,
        search_query,
        doc_search_active_index,
    );
    if let Some((global_index, new_state)) = actions.into_iter().next() {
        *action = AppAction::ToggleTaskList {
            global_index,
            new_state,
        };
    }
}
